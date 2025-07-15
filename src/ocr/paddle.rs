use std::error::Error;
use image::{DynamicImage, GenericImageView};
use rust_paddle_ocr::{Det, Rec};

pub struct PaddleOcr {
    d: Det,
    r: Rec,
}
impl PaddleOcr {
    pub fn new() -> Self {
        let mut det = Det::from_file("./models/PP-OCRv5_mobile_det.mnn").unwrap();
        let mut rec = Rec::from_file(
            "./models/PP-OCRv5_mobile_rec.mnn",
            "./models/ppocr_keys_v5.txt"
        ).unwrap();



        // 自定义检测参数（可选）
         det = det
            .with_rect_border_size(12)  // PP-OCRv5 推荐参数
            .with_merge_boxes(false)    // PP-OCRv5 推荐参数
            .with_merge_threshold(1);   // PP-OCRv5 推荐参数

        // 自定义识别参数（可选）
         rec = rec
            .with_min_score(0.6)
            .with_punct_min_score(0.1);

        Self{d:det, r:rec}
    }
}

impl super::Ocr for PaddleOcr {
    fn ocr(&mut self, img: &DynamicImage) -> Result<Vec<String>, Box<dyn Error>> {
        let mut result = Vec::with_capacity(4);
        let text_images = self.d.find_text_img(img)?;

        // 识别每个检测区域中的文本
        for text_img in text_images {
            result.push(self.r.predict_str(&text_img)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use crate::ocr::Ocr;
    use crate::ocr::paddle::PaddleOcr;

    #[test]
    fn orc() {
        let mut ocr = PaddleOcr::new();
        let img = image::open("./0.png").unwrap();
        println!("{:?}", ocr.ocr(&img))
    }
}
