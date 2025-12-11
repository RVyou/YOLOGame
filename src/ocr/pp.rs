use geo_clipper::{Clipper, EndType, JoinType};
use geo_types::{LineString, Polygon};

use image::{imageops, GrayImage, Luma, Rgb, RgbImage};
use imageproc::contours::Contour;
use imageproc::geometric_transformations::{warp_into, Interpolation, Projection};

use crate::ocr::{DET_MODEL_PATH, KEYS, REC_MODEL_PATH};
use ndarray::{Array, Array4};
use ort::{
    inputs,
    session::{
        builder::{GraphOptimizationLevel},
        Session,
    },
    value::Tensor,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct OcrItem {
    /// 文本框位置：使用元组 (x, y) 表示
    pub points: Vec<(u32, u32)>,
    /// 识别内容
    pub text: String,
    /// 置信度
    pub score: f32,
}

// --- 内部使用的结构 ---

#[derive(Debug, Clone)]
struct TextBox {
    // pub score: f32,
    pub points: Vec<(u32, u32)>, // 内部也改成元组
}

#[derive(Debug, Default, Clone)]
struct TextLine {
    pub text: String,
    pub text_score: f32,
}

const MEAN_VALS: [f32; 3] = [0.485 * 255.0, 0.456 * 255.0, 0.406 * 255.0];
const NORM_VALS: [f32; 3] = [
    1.0 / (0.229 * 255.0),
    1.0 / (0.224 * 255.0),
    1.0 / (0.225 * 255.0),
];
const CRNN_MEAN: [f32; 3] = [127.5, 127.5, 127.5];
const CRNN_NORM: [f32; 3] = [1.0 / 127.5, 1.0 / 127.5, 1.0 / 127.5];

struct OnnxModel {
    session: Session,
}

pub enum ModelSource<'a> {
    Path(&'a str),
    #[allow(dead_code)]
    Bytes(&'a [u8]),
}

impl OnnxModel {
    fn new(model_data: ModelSource, num_threads: usize) -> Result<Self> {
        let builder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level2)?
            .with_intra_threads(num_threads)?;

        let session = match model_data {
            ModelSource::Path(p) => builder.commit_from_file(p)?,
            ModelSource::Bytes(b) => builder.commit_from_memory(b)?,
        };
        Ok(Self { session })
    }

    fn run(&mut self, input: Array4<f32>) -> Result<ort::session::SessionOutputs> {
        let input_name = self.session.inputs[0].name.clone();
        let tensor = Tensor::from_array(input)?;
        Ok(self.session.run(inputs![input_name => tensor])?)
    }
}

// 图像处理工具类

struct OcrUtils;

impl OcrUtils {
    fn normalize(img: &RgbImage, mean: &[f32], norm: &[f32]) -> Array4<f32> {
        let (width, height) = img.dimensions();
        let mut array = Array::zeros((1, 3, height as usize, width as usize));

        for (x, y, pixel) in img.enumerate_pixels() {
            for c in 0..3 {
                let val = pixel[c] as f32;
                array[[0, c, y as usize, x as usize]] = (val - mean[c]) * norm[c];
            }
        }
        array
    }

    fn pad_image(img: &RgbImage, padding: u32) -> RgbImage {
        if padding == 0 {
            return img.clone();
        }
        let (w, h) = img.dimensions();
        let mut padded =
            RgbImage::from_pixel(w + 2 * padding, h + 2 * padding, Rgb([255, 255, 255]));
        imageops::replace(&mut padded, img, padding as i64, padding as i64);
        padded
    }

    /// 透视变换裁剪，输入为元组数组
    fn get_crop(img: &RgbImage, points: &[(u32, u32)]) -> RgbImage {
        let pts: Vec<(f32, f32)> = points.iter().map(|p| (p.0 as f32, p.1 as f32)).collect();
        if pts.len() != 4 {
            return img.clone();
        }

        let w = ((pts[0].0 - pts[1].0).hypot(pts[0].1 - pts[1].1)) as u32;
        let h = ((pts[0].0 - pts[3].0).hypot(pts[0].1 - pts[3].1)) as u32;

        let src = [pts[0], pts[1], pts[2], pts[3]];
        let dst = [
            (0.0, 0.0),
            (w as f32, 0.0),
            (w as f32, h as f32),
            (0.0, h as f32),
        ];

        if w == 0 || h == 0 {
            return img.clone();
        }

        let proj = Projection::from_control_points(src, dst).unwrap_or(Projection::scale(1.0, 1.0));
        let mut out = RgbImage::new(w, h);
        warp_into(
            img,
            &proj,
            Interpolation::Nearest,
            Rgb([255, 255, 255]),
            &mut out,
        );

        if out.height() as f32 / out.width() as f32 > 1.5 {
            imageops::rotate90(&out)
        } else {
            out
        }
    }
}

// =======================================================
// 文本检测)
// =======================================================
struct DbNet {
    model: Option<OnnxModel>,
}

impl DbNet {
    fn forward(&mut self, img: &RgbImage, params: &DetectParams) -> Result<Vec<TextBox>> {
        let Some(model) = &mut self.model else {
            return Err("DBNet model not initialized".into());
        };

        // Resize
        let (w, h) = img.dimensions();
        let max_side = params.max_side_len.min(w.max(h)) + 2 * params.padding;
        let ratio = max_side as f32 / w.max(h) as f32;
        let (mut resize_w, mut resize_h) = ((w as f32 * ratio) as u32, (h as f32 * ratio) as u32);

        resize_w = (resize_w / 32).max(1) * 32;
        resize_h = (resize_h / 32).max(1) * 32;

        let resized = imageops::resize(img, resize_w, resize_h, imageops::FilterType::Triangle);
        let ratio_w = resize_w as f32 / w as f32;
        let ratio_h = resize_h as f32 / h as f32;

        // Inference
        let input = OcrUtils::normalize(&resized, &MEAN_VALS, &NORM_VALS);
        let output = model.run(input)?;

        let (shape, data) = output[0].try_extract_tensor::<f32>()?;
        let out_h = shape[2] as usize;
        let out_w = shape[3] as usize;

        // Post-processing
        let bitmap: Vec<u8> = data
            .iter()
            .map(|&x| if x > params.box_thresh { 255 } else { 0 })
            .collect();
        let gray_img = GrayImage::from_vec(out_w as u32, out_h as u32, bitmap)
            .ok_or("Failed to create bitmap image")?;

        let contours = imageproc::contours::find_contours(&gray_img);
        let mut boxes = Vec::new();

        for contour in contours {
            if contour.points.len() < 3 {
                continue;
            }
            let score = Self::box_score(&contour, data, out_w, out_h);
            if score < params.box_score_thresh {
                continue;
            }

            // 计算外接矩形 (ImageProc 内部仍使用 Point<i32> 进行计算，这里仅在最终输出转为元组)
            let raw_box = Self::get_mini_box_points(&contour.points);
            if Self::side_len(&raw_box) < 3.0 {
                continue;
            }

            let unclipped_poly = Self::unclip(&raw_box, params.unclip_ratio);
            if unclipped_poly.len() < 3 {
                continue;
            }

            let rect_points = Self::get_mini_box_points(&unclipped_poly);
            if Self::side_len(&rect_points) < 3.0 {
                continue;
            }

            // 映射回原图坐标，并转换为元组 (x, y)
            let final_points: Vec<(u32, u32)> = rect_points
                .iter()
                .map(|p| {
                    (
                        (p.x / ratio_w).min(w as f32) as u32,
                        (p.y / ratio_h).min(h as f32) as u32,
                    )
                })
                .collect();

            boxes.push(TextBox {
                // score,
                points: final_points,
            });
        }
        Ok(boxes)
    }

    fn box_score(contour: &Contour<i32>, pred: &[f32], w: usize, h: usize) -> f32 {
        let (xmin, ymin, xmax, ymax) = contour.points.iter().fold((w, h, 0, 0), |acc, p| {
            (
                acc.0.min(p.x as usize),
                acc.1.min(p.y as usize),
                acc.2.max(p.x as usize),
                acc.3.max(p.y as usize),
            )
        });
        let roi_w = xmax - xmin + 1;
        let roi_h = ymax - ymin + 1;
        if roi_w == 0 || roi_h == 0 {
            return 0.0;
        }

        let mut mask = GrayImage::new(roi_w as u32, roi_h as u32);
        let pts: Vec<_> = contour
            .points
            .iter()
            .map(|p| imageproc::point::Point::new(p.x - xmin as i32, p.y - ymin as i32))
            .collect();
        imageproc::drawing::draw_polygon_mut(&mut mask, &pts, Luma([1]));

        let mut sum = 0.0;
        let mut count = 0;
        for y in 0..roi_h {
            for x in 0..roi_w {
                if mask.get_pixel(x as u32, y as u32)[0] > 0 {
                    let idx = (ymin + y) * w + (xmin + x);
                    if idx < pred.len() {
                        sum += pred[idx];
                        count += 1;
                    }
                }
            }
        }
        if count == 0 {
            0.0
        } else {
            sum / count as f32
        }
    }

    fn polygon_area(pts: &[imageproc::point::Point<f32>]) -> f32 {
        if pts.is_empty() {
            return 0.0;
        }
        pts.iter()
            .zip(pts.iter().cycle().skip(1))
            .map(|(p0, p1)| (p1.x - p0.x) * (p1.y + p0.y))
            .sum::<f32>()
            / 2.0
    }

    fn polygon_len(pts: &[imageproc::point::Point<f32>]) -> f32 {
        if pts.is_empty() {
            return 0.0;
        }
        pts.iter()
            .zip(pts.iter().cycle().skip(1))
            .map(|(p0, p1)| ((p0.x - p1.x).powi(2) + (p0.y - p1.y).powi(2)).sqrt())
            .sum()
    }

    fn get_mini_box_points(
        pts: &[imageproc::point::Point<i32>],
    ) -> Vec<imageproc::point::Point<f32>> {
        let rect = imageproc::geometry::min_area_rect(pts);
        rect.iter()
            .map(|p| imageproc::point::Point::new(p.x as f32, p.y as f32))
            .collect()
    }

    fn unclip(
        points: &[imageproc::point::Point<f32>],
        ratio: f32,
    ) -> Vec<imageproc::point::Point<i32>> {
        let area = Self::polygon_area(points).abs();
        let len = Self::polygon_len(points);
        if len == 0.0 {
            return vec![];
        }
        let distance = area * ratio / len;

        let poly = Polygon::new(
            LineString::new(
                points
                    .iter()
                    .map(|p| geo_types::Coord {
                        x: p.x as f64,
                        y: p.y as f64,
                    })
                    .collect(),
            ),
            vec![],
        );
        let offset = poly.offset(
            distance as f64,
            JoinType::Round(2.0),
            EndType::ClosedPolygon,
            1.0,
        );

        offset
            .0
            .first()
            .map(|p| {
                p.exterior()
                    .points()
                    .map(|c| imageproc::point::Point::new(c.x() as i32, c.y() as i32))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn side_len(pts: &[imageproc::point::Point<f32>]) -> f32 {
        if pts.len() < 3 {
            return 0.0;
        }
        let w = ((pts[0].x - pts[1].x).powi(2) + (pts[0].y - pts[1].y).powi(2)).sqrt();
        let h = ((pts[1].x - pts[2].x).powi(2) + (pts[1].y - pts[2].y).powi(2)).sqrt();
        w.min(h)
    }
}

// =======================================================
// 子网络文本识别
// =======================================================
struct CrnnNet {
    model: Option<OnnxModel>,
    keys: Vec<String>,
}

impl CrnnNet {
    fn load_keys(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.keys = content.lines().map(|s| s.to_string()).collect();
        self.keys.insert(0, "#".into()); // Blank
        self.keys.push(" ".into());
        Ok(())
    }

    fn predict(&mut self, img: &RgbImage) -> Result<TextLine> {
        let Some(model) = &mut self.model else {
            return Err("CrnnNet model not initialized".into());
        };

        let scale = 48.0 / img.height() as f32;
        let dst_w = (img.width() as f32 * scale) as u32;
        let input_img = imageops::resize(img, dst_w, 48, imageops::FilterType::Triangle);
        let tensor = OcrUtils::normalize(&input_img, &CRNN_MEAN, &CRNN_NORM);

        let out = model.run(tensor)?;
        let (shape, data) = out[0].try_extract_tensor::<f32>()?;

        let seq_len = shape[1] as usize;
        let num_classes = shape[2] as usize;

        let mut raw_str = String::new();
        let mut score_sum = 0.0;
        let mut valid_chars = 0;
        let mut last_idx = 0;

        for i in 0..seq_len {
            let start = i * num_classes;
            let slice = &data[start..start + num_classes];

            let (max_idx, max_val) = slice
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap_or((0, &0.0));

            if max_idx > 0 && max_idx < self.keys.len() && (max_idx != last_idx) {
                raw_str.push_str(&self.keys[max_idx]);
                score_sum += max_val;
                valid_chars += 1;
            }
            last_idx = max_idx;
        }

        Ok(TextLine {
            text: raw_str,
            text_score: if valid_chars > 0 {
                score_sum / valid_chars as f32
            } else {
                0.0
            },
        })
    }
}

// =======================================================
// 主入口
// =======================================================

#[derive(Debug, Clone)]
pub struct DetectParams {
    pub padding: u32,
    pub max_side_len: u32,
    pub box_score_thresh: f32,
    pub box_thresh: f32,
    pub unclip_ratio: f32,
}

impl Default for DetectParams {
    fn default() -> Self {
        Self {
            padding: 50,
            max_side_len: 960,
            box_score_thresh: 0.5,
            box_thresh: 0.3,
            unclip_ratio: 1.5,
        }
    }
}

pub struct OcrLite {
    db: DbNet,
    crnn: CrnnNet,
    num_threads: usize,
}

impl OcrLite {
    pub fn new(num_threads: usize) -> Self {
        Self {
            db: DbNet { model: None },
            crnn: CrnnNet {
                model: None,
                keys: vec![],
            },
            num_threads,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.db.model = Some(OnnxModel::new(
            ModelSource::Path(DET_MODEL_PATH),
            self.num_threads,
        )?);
        self.crnn.model = Some(OnnxModel::new(
            ModelSource::Path(REC_MODEL_PATH),
            self.num_threads,
        )?);
        self.crnn.load_keys(KEYS)?;
        Ok(())
    }


    pub fn detect(&mut self, img: &RgbImage, params: DetectParams) -> Result<Vec<OcrItem>> {
        // 1. Padding
        let padded_img = OcrUtils::pad_image(img, params.padding);

        // 2. DBNet Detect
        let boxes = self.db.forward(&padded_img, &params)?;

        // 3. Crop & Recognize
        let mut results = Vec::with_capacity(boxes.len());

        for b in boxes {
            let part_img = OcrUtils::get_crop(&padded_img, &b.points);

            // 4. CRNN Rec
            let rec_res = self.crnn.predict(&part_img)?;

            // 5. 还原坐标 & 组装结果
            let real_points: Vec<(u32, u32)> = b
                .points
                .iter()
                .map(|p| {
                    (
                        p.0.saturating_sub(params.padding),
                        p.1.saturating_sub(params.padding),
                    )
                })
                .collect();

            results.push(OcrItem {
                points: real_points,
                text: rec_res.text,
                score: rec_res.text_score,
            });
        }

        Ok(results)
    }
}
