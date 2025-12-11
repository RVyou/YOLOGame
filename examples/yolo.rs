use lib::yolo::model::YoloDetector;

fn main() {


    lib::onnx_init();

    let image_path = "examples/cats.jpg";
    let model_path = "extra/yolov8s.onnx";

    let mut detector = YoloDetector::new(model_path, 0.4, 0.45).unwrap();

    println!("读取图片...");
    let img = image::open(image_path).expect("Failed to open image");


    println!("开始检测...");
    let detections = detector.detect(&img).unwrap();


    println!("检测到 {} 个目标:", detections.len());
    for d in detections {
        println!(
            "目标: {}, 置信度: {:.2}%, 坐标: ({:.1}, {:.1}) - ({:.1}, {:.1})",
            d.label,
            d.confidence * 100.0,
            d.x1,
            d.y1,
            d.x2,
            d.y2
        );
    }
}
