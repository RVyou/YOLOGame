//! # Rust PaddleOCR
//! 
//! 基于 PaddleOCR 模型的 Rust OCR 库，提供轻量级、高效的文字识别功能。
//! A Rust OCR library based on PaddleOCR models, providing lightweight and efficient text recognition.
//! 
//! ## 主要功能 (Main Features)
//! 
//! - 文本检测 (Text Detection): 定位图像中的文本区域
//! - 文本识别 (Text Recognition): 识别检测到的文本内容
//! - 线程安全的OCR引擎 (Thread-safe OCR Engine): 多线程环境下安全使用OCR功能
//! 
//! ## 使用示例 (Usage Example)
//! 
//! ```rust,no_run
//! use rust_paddle_ocr::{OcrEngineManager};
//! use image::open;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 初始化OCR引擎
//!     // Initialize OCR engine
//!     OcrEngineManager::initialize("path/to/det_model.mnn", "path/to/rec_model.mnn", "path/to/keys.txt")?;
//!     
//!     // 打开图像
//!     // Open image
//!     let img = open("path/to/image.jpg")?;
//!     
//!     // 处理OCR，自动检测文本区域并识别
//!     // Process OCR, automatically detect text regions and recognize
//!     let texts = OcrEngineManager::process_ocr(img)?;
//!     
//!     // 打印识别结果
//!     // Print recognition results
//!     for text in texts {
//!         println!("Recognized text: {}", text);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//! 
//! 对于需要直接使用模型的高级用户，仍然可以访问底层 API：
//! For advanced users who need to directly use the models, the underlying API is still accessible:
//! 
//! ```rust,no_run
//! use rust_paddle_ocr::{Det, Rec};
//! use image::open;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 加载检测模型
//!     // Load detection model
//!     let mut det = Det::from_file("path/to/det_model.mnn")?;
//!     
//!     // 加载识别模型
//!     // Load recognition model
//!     let mut rec = Rec::from_file("path/to/rec_model.mnn", "path/to/keys.txt")?;
//!     
//!     // 打开图像
//!     // Open image
//!     let img = open("path/to/image.jpg")?;
//!     
//!     // 检测文本区域
//!     // Detect text regions
//!     let text_images = det.find_text_img(&img)?;
//!     
//!     // 识别每个文本区域的内容
//!     // Recognize content in each text region
//!     for text_img in text_images {
//!         let text = rec.predict_str(&text_img)?;
//!         println!("Recognized text: {}", text);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod det;
pub mod engine;
pub mod error;
pub mod rec;

// C API 模块
// C API module
// pub mod capi;

pub use det::Det;
pub use engine::{OcrEngine, OcrEngineManager};
pub use error::{OcrError, OcrResult};
pub use rec::Rec;

// 重新导出 C API
// Re-export C API
// pub use capi::*;
