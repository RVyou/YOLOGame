use image::{ImageBuffer, Luma};
use template_matching::{find_extremes, MatchTemplateMethod, TemplateMatcher};

pub mod adb;

pub mod yolo;
pub mod ocr;

pub fn onnx_init() {
    ort::init_from("extra/onnxruntime1221.dll")
        .commit()
        .expect("ort::init_from error");
}
pub fn match_template(
    input_image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template_image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    matcher: &mut TemplateMatcher,
) -> template_matching::Extremes {
    matcher.match_template(
        input_image,
        template_image,
        MatchTemplateMethod::SumOfSquaredDifferences,
    );
    let result = matcher.wait_for_result().unwrap();

    let extremes = find_extremes(&result);
    extremes
}

//Ok(ocr_result) = self.paddle_ocr.ocr(&data_image)
// paddle_ocr: ocr::paddle::PaddleOcr::new(),
// thread::sleep(time::Duration::from_millis(1800));
// fn x(&mut self, v: &str) {
//     println!("游戏关闭x窗口");
//     let mut screencap = screencap_device(v).to_luma32f();
//     let result = match_template(&screencap, &self.comment.x_image, &mut self.matcher);
//     if result.min_value < 13_f32 {
//         adb_Click(
//             v,
//             &result.min_value_location.0.to_string(),
//             &result.min_value_location.1.to_string(),
//         );
//         println!("找到游戏x窗口{:?},点击关闭", result.min_value_location);
//         while match_template(&screencap, &self.comment.x_image, &mut self.matcher).min_value
//             > 13_f32
//         {
//             thread::sleep(time::Duration::from_millis(500));
//             screencap = screencap_device(v).to_luma32f();
//         }
//     } else {
//         println!("未找到游戏内x窗口");
//     }
// }
