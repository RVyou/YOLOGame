use lib::ocr::pp::{DetectParams, OcrLite};

fn main() {
    lib::onnx_init();

    let img_path = "examples/pp_test.png";

    let mut ocr = OcrLite::new(3);

    if let Err(e) = ocr.init() {
        eprintln!("模型初始化失败: {}", e);
        return;
    }

    println!("开始识别...");
    match ocr.detect(&image::open(img_path).unwrap().to_rgb8(), DetectParams::default()) {
        Ok(items) => {
            for (_i, item) in items.iter().enumerate() {
                if item.score < 0.5 {
                    continue;
                }
                println!(" {:?}", item.text);
            }
        }
        Err(e) => eprintln!("识别出错: {}", e),
    }
}
