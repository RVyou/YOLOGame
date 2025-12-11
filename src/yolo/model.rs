use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use ndarray::{s, Array, ArrayView, Axis, Ix3, IxDyn};
use ort::{
    execution_providers::{CUDAExecutionProvider, TensorRTExecutionProvider},
    inputs,
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Detection {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub label: String,
    pub confidence: f32,
}

pub struct YoloDetector {
    session: Session,
    classes: Vec<String>,
    input_width: u32,
    input_height: u32,
    conf_threshold: f32,
    iou_threshold: f32,
}

impl YoloDetector {
    pub fn new<P: AsRef<Path>>(
        model_path: P,
        conf_threshold: f32,
        iou_threshold: f32,
    ) -> Result<Self> {

        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .with_execution_providers([
                TensorRTExecutionProvider::default().build(),
                CUDAExecutionProvider::default().build(),
            ])?
            .commit_from_file(model_path)?;

        let mut result = Self {
            session,
            classes: vec![],
            input_width: 640,
            input_height: 640,
            conf_threshold,
            iou_threshold,
        };
        let classes = result.names().unwrap();

        result.classes = classes;
        Ok(result)
    }

    pub fn fetch_from_metadata(&self, key: &str) -> Option<String> {
        match self.session.metadata() {
            Err(_) => None,
            Ok(metadata) => metadata.custom(key).unwrap_or_else(|_| None),
        }
    }

    pub fn names(&self) -> Option<Vec<String>> {
        match self.fetch_from_metadata("names") {
            Some(names) => {
                let re = Regex::new(r#"(['"])([-()\w '"]+)(['"])"#).unwrap();
                let mut names_ = vec![];
                for (_, [_, name, _]) in re.captures_iter(&names).map(|x| x.extract()) {
                    names_.push(name.to_string());
                }
                Some(names_)
            }
            None => None,
        }
    }

    pub fn detect(&mut self, img: &DynamicImage) -> Result<Vec<Detection>> {
        //  预处理 (不再需要在函数内加载文件)
        let (input_tensor, original_w, original_h) = self.prepare_input(img);

        //  推理与数据转换
        let output_array = {
            let outputs = self.session.run(inputs!["images" => input_tensor])?;

            let output_tensor = outputs["output0"].try_extract_tensor::<f32>()?;
            let (shape_ref, data_slice) = output_tensor;

            let shape_usize: Vec<usize> = shape_ref.iter().map(|&x| x as usize).collect();

            let output_view_dyn = ArrayView::from_shape(IxDyn(&shape_usize), data_slice)?;

            let output_view_3d = output_view_dyn
                .into_dimensionality::<Ix3>()
                .context("模型输出形状不符合预期 (应为 3 维)")?;

            output_view_3d.permuted_axes([0, 2, 1]).to_owned()
        };

        //  后处理
        let detections = self.process_output(output_array, original_w, original_h);

        Ok(detections)
    }


    fn prepare_input(&self, img: &DynamicImage) -> (Tensor<f32>, u32, u32) {
        let (img_width, img_height) = (img.width(), img.height());
        let img_resized =
            img.resize_exact(self.input_width, self.input_height, FilterType::CatmullRom);

        let mut input = Array::zeros((1, 3, self.input_height as usize, self.input_width as usize));

        for pixel in img_resized.pixels() {
            let x = pixel.0 as usize;
            let y = pixel.1 as usize;
            let [r, g, b, _] = pixel.2 .0;
            input[[0, 0, y, x]] = (r as f32) / 255.0;
            input[[0, 1, y, x]] = (g as f32) / 255.0;
            input[[0, 2, y, x]] = (b as f32) / 255.0;
        }

        let tensor = Tensor::from_array(input).unwrap();
        (tensor, img_width, img_height)
    }

    fn process_output(
        &self,
        output: Array<f32, Ix3>,
        img_width: u32,
        img_height: u32,
    ) -> Vec<Detection> {
        let mut boxes = Vec::new();
        let output_2d = output.slice(s![0, .., ..]);

        for row in output_2d.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().map(|x| *x).collect();

            let (class_id, prob) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap_or((0, 0.0));

            if prob < self.conf_threshold {
                continue;
            }

            let label = self
                .classes
                .get(class_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let xc = row[0] / (self.input_width as f32) * (img_width as f32);
            let yc = row[1] / (self.input_height as f32) * (img_height as f32);
            let w = row[2] / (self.input_width as f32) * (img_width as f32);
            let h = row[3] / (self.input_height as f32) * (img_height as f32);

            let x1 = xc - w / 2.0;
            let x2 = xc + w / 2.0;
            let y1 = yc - h / 2.0;
            let y2 = yc + h / 2.0;

            boxes.push(Detection {
                x1,
                y1,
                x2,
                y2,
                label,
                confidence: prob,
            });
        }

        boxes.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
        let mut result = Vec::new();
        while !boxes.is_empty() {
            let current = boxes.remove(0);
            result.push(current.clone());
            boxes.retain(|box1| iou(&current, box1) < self.iou_threshold);
        }
        result
    }
}

fn iou(box1: &Detection, box2: &Detection) -> f32 {
    let inter = intersection(box1, box2);
    let u = union_area(box1, box2, inter);
    if u == 0.0 {
        0.0
    } else {
        inter / u
    }
}

fn intersection(box1: &Detection, box2: &Detection) -> f32 {
    let x1 = box1.x1.max(box2.x1);
    let y1 = box1.y1.max(box2.y1);
    let x2 = box1.x2.min(box2.x2);
    let y2 = box1.y2.min(box2.y2);
    if x2 < x1 || y2 < y1 {
        return 0.0;
    }
    (x2 - x1) * (y2 - y1)
}

fn union_area(box1: &Detection, box2: &Detection, inter_area: f32) -> f32 {
    let area1 = (box1.x2 - box1.x1) * (box1.y2 - box1.y1);
    let area2 = (box2.x2 - box2.x1) * (box2.y2 - box2.y1);
    area1 + area2 - inter_area
}