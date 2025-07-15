use crate::{Det, OcrError, OcrResult, Rec};
use crossbeam_channel::{unbounded, Receiver, Sender};
use image::DynamicImage;
use imageproc::rect::Rect;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

/// OCR请求类型
///
/// Types of OCR requests
#[derive(Debug)]
pub enum OcrRequest {
    /// 文本检测请求
    /// Text detection request
    DetectText {
        /// 输入图像
        /// Input image
        image: DynamicImage,
        /// 结果发送通道
        /// Result sender channel
        result_sender: Sender<OcrResult<Vec<DynamicImage>>>,
    },
    /// 文本识别请求
    /// Text recognition request
    RecognizeText {
        /// 输入图像
        /// Input image
        image: DynamicImage,
        /// 结果发送通道
        /// Result sender channel
        result_sender: Sender<OcrResult<String>>,
    },
    /// 完整OCR处理请求
    /// Full OCR processing request
    ProcessOcr {
        /// 输入图像
        /// Input image
        image: DynamicImage,
        /// 结果发送通道
        /// Result sender channel
        result_sender: Sender<OcrResult<Vec<String>>>,
    },
    /// 获取文本区域矩形框请求
    /// Get text region rectangles request
    GetTextRects {
        /// 输入图像
        /// Input image
        image: DynamicImage,
        /// 结果发送通道
        /// Result sender channel
        result_sender: Sender<OcrResult<Vec<Rect>>>,
    },
    /// 获取文本区域图像请求
    /// Get text region images request
    GetTextImages {
        /// 输入图像
        /// Input image
        image: DynamicImage,
        /// 结果发送通道
        /// Result sender channel
        result_sender: Sender<OcrResult<Vec<DynamicImage>>>,
    },
    /// 关闭引擎请求
    /// Shutdown engine request
    Shutdown,
}

/// 线程安全的OCR引擎管理器
///
/// Thread-safe OCR engine manager
pub struct OcrEngine {
    request_sender: Sender<OcrRequest>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl OcrEngine {
    /// 创建并启动一个新的OCR引擎实例
    ///
    /// Create and start a new OCR engine instance
    pub fn new(
        det_model_path: impl AsRef<Path>,
        rec_model_path: impl AsRef<Path>,
        keys_path: impl AsRef<Path>,
    ) -> OcrResult<Self> {
        Self::new_with_config(
            det_model_path,
            rec_model_path,
            keys_path,
            Det::RECT_BORDER_SIZE,
            false,
            Det::DEFAULT_MERGE_THRESHOLD,
        )
    }

    /// 创建并启动一个带有自定义配置的OCR引擎实例
    ///
    /// Create and start a new OCR engine instance with custom configuration
    pub fn new_with_config(
        det_model_path: impl AsRef<Path>,
        rec_model_path: impl AsRef<Path>,
        keys_path: impl AsRef<Path>,
        rect_border_size: u32,
        merge_boxes: bool,
        merge_threshold: i32,
    ) -> OcrResult<Self> {
        // 创建通信通道
        let (tx, rx) = unbounded();

        // 创建工作线程，该线程将持有OCR模型
        let worker_handle = thread::spawn({
            let det_path = det_model_path.as_ref().to_path_buf();
            let rec_path = rec_model_path.as_ref().to_path_buf();
            let keys = keys_path.as_ref().to_path_buf();

            move || match Self::run_worker(
                det_path,
                rec_path,
                keys,
                rx,
                rect_border_size,
                merge_boxes,
                merge_threshold,
            ) {
                Ok(_) => {}
                Err(e) => eprintln!("OCR worker error: {}", e),
            }
        });

        Ok(Self {
            request_sender: tx,
            worker_handle: Some(worker_handle),
        })
    }

    /// 创建并启动一个带有自定义配置和字节数据的OCR引擎实例
    ///
    /// Create and start a new OCR engine instance with custom configuration and byte data
    pub fn new_with_config_and_bytes(
        det_model_data: &[u8],
        rec_model_data: &[u8],
        keys_data: &[u8],
        rect_border_size: u32,
        merge_boxes: bool,
        merge_threshold: i32,
    ) -> OcrResult<Self> {
        // 创建通信通道
        let (tx, rx) = unbounded();

        // 克隆字节数据，准备传递给工作线程
        let det_data = det_model_data.to_vec();
        let rec_data = rec_model_data.to_vec();
        let keys = keys_data.to_vec();

        // 创建工作线程，该线程将持有OCR模型
        let worker_handle = thread::spawn(move || {
            match Self::run_worker_with_bytes(
                det_data,
                rec_data,
                keys,
                rx,
                rect_border_size,
                merge_boxes,
                merge_threshold,
            ) {
                Ok(_) => {}
                Err(e) => eprintln!("OCR worker error: {}", e),
            }
        });

        Ok(Self {
            request_sender: tx,
            worker_handle: Some(worker_handle),
        })
    }

    /// 在图像中检测文本区域
    ///
    /// Detect text regions in the image
    pub fn detect_text(&self, image: DynamicImage) -> OcrResult<Vec<DynamicImage>> {
        // 创建结果通道
        let (result_tx, result_rx) = unbounded();

        // 发送请求
        self.request_sender
            .send(OcrRequest::DetectText {
                image,
                result_sender: result_tx,
            })
            .map_err(|_| {
                OcrError::EngineError("OCR engine worker thread has terminated".to_string())
            })?;

        // 等待结果
        result_rx.recv().map_err(|_| {
            OcrError::EngineError("Failed to receive result from worker thread".to_string())
        })?
    }

    /// 获取文本区域的矩形框
    ///
    /// Get text region rectangles
    pub fn get_text_rects(&self, image: &DynamicImage) -> OcrResult<Vec<Rect>> {
        // 创建结果通道
        let (result_tx, result_rx) = unbounded();

        // 发送请求
        self.request_sender
            .send(OcrRequest::GetTextRects {
                image: image.clone(),
                result_sender: result_tx,
            })
            .map_err(|_| {
                OcrError::EngineError("OCR engine worker thread has terminated".to_string())
            })?;

        // 等待结果
        result_rx.recv().map_err(|_| {
            OcrError::EngineError("Failed to receive result from worker thread".to_string())
        })?
    }

    /// 获取文本区域图像
    ///
    /// Get text region images
    pub fn get_text_images(&self, image: &DynamicImage) -> OcrResult<Vec<DynamicImage>> {
        // 创建结果通道
        let (result_tx, result_rx) = unbounded();

        // 发送请求
        self.request_sender
            .send(OcrRequest::GetTextImages {
                image: image.clone(),
                result_sender: result_tx,
            })
            .map_err(|_| {
                OcrError::EngineError("OCR engine worker thread has terminated".to_string())
            })?;

        // 等待结果
        result_rx.recv().map_err(|_| {
            OcrError::EngineError("Failed to receive result from worker thread".to_string())
        })?
    }

    /// 识别图像中的文本
    ///
    /// Recognize text in the image
    pub fn recognize_text(&self, image: DynamicImage) -> OcrResult<String> {
        // 创建结果通道
        let (result_tx, result_rx) = unbounded();

        // 发送请求
        self.request_sender
            .send(OcrRequest::RecognizeText {
                image,
                result_sender: result_tx,
            })
            .map_err(|_| {
                OcrError::EngineError("OCR engine worker thread has terminated".to_string())
            })?;

        // 等待结果
        result_rx.recv().map_err(|_| {
            OcrError::EngineError("Failed to receive result from worker thread".to_string())
        })?
    }

    /// 完整的OCR处理，检测并识别图像中的所有文本
    ///
    /// Complete OCR processing, detecting and recognizing all text in the image
    pub fn process_ocr(&self, image: DynamicImage) -> OcrResult<Vec<String>> {
        // 创建结果通道
        let (result_tx, result_rx) = unbounded();

        // 发送请求
        self.request_sender
            .send(OcrRequest::ProcessOcr {
                image,
                result_sender: result_tx,
            })
            .map_err(|_| {
                OcrError::EngineError("OCR engine worker thread has terminated".to_string())
            })?;

        // 等待结果
        result_rx.recv().map_err(|_| {
            OcrError::EngineError("Failed to receive result from worker thread".to_string())
        })?
    }

    /// 工作线程的主处理函数
    ///
    /// Main processing function for the worker thread
    fn run_worker(
        det_model_path: impl AsRef<Path>,
        rec_model_path: impl AsRef<Path>,
        keys_path: impl AsRef<Path>,
        receiver: Receiver<OcrRequest>,
        rect_border_size: u32,
        merge_boxes: bool,
        merge_threshold: i32,
    ) -> OcrResult<()> {
        // 初始化模型，应用自定义配置
        let mut det = Det::from_file(det_model_path)?
            .with_rect_border_size(rect_border_size)
            .with_merge_boxes(merge_boxes)
            .with_merge_threshold(merge_threshold);

        let mut rec = Rec::from_file(rec_model_path, keys_path)?;

        // 处理请求循环
        for request in receiver {
            match request {
                OcrRequest::DetectText {
                    image,
                    result_sender,
                } => {
                    let result = det.find_text_img(&image);
                    // 发送结果，忽略接收端可能已关闭的错误
                    let _ = result_sender.send(result);
                }
                OcrRequest::GetTextRects {
                    image,
                    result_sender,
                } => {
                    let result = det.find_text_rect(&image);
                    let _ = result_sender.send(result);
                }
                OcrRequest::GetTextImages {
                    image,
                    result_sender,
                } => {
                    let result = det.find_text_img(&image);
                    let _ = result_sender.send(result);
                }
                OcrRequest::RecognizeText {
                    image,
                    result_sender,
                } => {
                    let result = rec.predict_str(&image);
                    let _ = result_sender.send(result);
                }
                OcrRequest::ProcessOcr {
                    image,
                    result_sender,
                } => {
                    // 先检测文本区域
                    match det.find_text_img(&image) {
                        Ok(text_images) => {
                            // 识别每个文本区域
                            let mut results = Vec::with_capacity(text_images.len());
                            for text_img in text_images {
                                match rec.predict_str(&text_img) {
                                    Ok(text) => results.push(text),
                                    Err(e) => {
                                        let _ = result_sender.send(Err(e));
                                        break;
                                    }
                                }
                            }
                            let _ = result_sender.send(Ok(results));
                        }
                        Err(e) => {
                            let _ = result_sender.send(Err(e));
                        }
                    }
                }
                OcrRequest::Shutdown => {
                    // 收到关闭请求，退出循环
                    break;
                }
            }
        }

        Ok(())
    }

    /// 使用字节数据的工作线程的主处理函数
    ///
    /// Main processing function for the worker thread using byte data
    fn run_worker_with_bytes(
        det_model_data: Vec<u8>,
        rec_model_data: Vec<u8>,
        keys_data: Vec<u8>,
        receiver: Receiver<OcrRequest>,
        rect_border_size: u32,
        merge_boxes: bool,
        merge_threshold: i32,
    ) -> OcrResult<()> {
        // 直接从字节数据初始化模型
        let mut det = Det::from_bytes(&det_model_data)?
            .with_rect_border_size(rect_border_size)
            .with_merge_boxes(merge_boxes)
            .with_merge_threshold(merge_threshold);

        let mut rec = Rec::from_bytes_with_keys(&rec_model_data, &keys_data)?;

        // 处理请求循环
        for request in receiver {
            match request {
                OcrRequest::DetectText {
                    image,
                    result_sender,
                } => {
                    let result = det.find_text_img(&image);
                    // 发送结果，忽略接收端可能已关闭的错误
                    let _ = result_sender.send(result);
                }
                OcrRequest::GetTextRects {
                    image,
                    result_sender,
                } => {
                    let result = det.find_text_rect(&image);
                    let _ = result_sender.send(result);
                }
                OcrRequest::GetTextImages {
                    image,
                    result_sender,
                } => {
                    let result = det.find_text_img(&image);
                    let _ = result_sender.send(result);
                }
                OcrRequest::RecognizeText {
                    image,
                    result_sender,
                } => {
                    let result = rec.predict_str(&image);
                    let _ = result_sender.send(result);
                }
                OcrRequest::ProcessOcr {
                    image,
                    result_sender,
                } => {
                    // 先检测文本区域
                    match det.find_text_img(&image) {
                        Ok(text_images) => {
                            // 识别每个文本区域
                            let mut results = Vec::with_capacity(text_images.len());
                            for text_img in text_images {
                                match rec.predict_str(&text_img) {
                                    Ok(text) => results.push(text),
                                    Err(e) => {
                                        let _ = result_sender.send(Err(e));
                                        break;
                                    }
                                }
                            }
                            let _ = result_sender.send(Ok(results));
                        }
                        Err(e) => {
                            let _ = result_sender.send(Err(e));
                        }
                    }
                }
                OcrRequest::Shutdown => {
                    // 收到关闭请求，退出循环
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Drop for OcrEngine {
    fn drop(&mut self) {
        // 发送关闭请求
        let _ = self.request_sender.send(OcrRequest::Shutdown);

        // 等待工作线程完成
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

/// 全局OCR引擎单例
///
/// Global OCR engine singleton
pub struct OcrEngineManager {
    // 私有构造函数，防止直接实例化
    _private: (),
}

// 全局单例实例，使用 Arc<Mutex<>> 确保线程安全
static INSTANCE: once_cell::sync::OnceCell<Arc<Mutex<Option<OcrEngine>>>> =
    once_cell::sync::OnceCell::new();

impl OcrEngineManager {
    /// 初始化全局OCR引擎
    ///
    /// Initialize the global OCR engine
    pub fn initialize(
        det_model_path: impl AsRef<Path>,
        rec_model_path: impl AsRef<Path>,
        keys_path: impl AsRef<Path>,
    ) -> OcrResult<()> {
        let engine = OcrEngine::new(det_model_path, rec_model_path, keys_path)?;

        // 获取或初始化全局实例
        let instance = INSTANCE.get_or_init(|| Arc::new(Mutex::new(None)));

        // 更新引擎实例
        let mut guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        *guard = Some(engine);

        Ok(())
    }

    /// 使用自定义配置初始化全局OCR引擎
    ///
    /// Initialize the global OCR engine with custom configuration
    pub fn initialize_with_config(
        det_model_path: impl AsRef<Path>,
        rec_model_path: impl AsRef<Path>,
        keys_path: impl AsRef<Path>,
        rect_border_size: u32,
        merge_boxes: bool,
        merge_threshold: i32,
    ) -> OcrResult<()> {
        let engine = OcrEngine::new_with_config(
            det_model_path,
            rec_model_path,
            keys_path,
            rect_border_size,
            merge_boxes,
            merge_threshold,
        )?;

        // 获取或初始化全局实例
        let instance = INSTANCE.get_or_init(|| Arc::new(Mutex::new(None)));

        // 更新引擎实例
        let mut guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        *guard = Some(engine);

        Ok(())
    }

    /// 使用自定义配置和字节数据初始化全局OCR引擎
    ///
    /// Initialize the global OCR engine with custom configuration and byte data
    pub fn initialize_with_config_and_bytes(
        det_model_data: &[u8],
        rec_model_data: &[u8],
        keys_data: &[u8],
        rect_border_size: u32,
        merge_boxes: bool,
        merge_threshold: i32,
    ) -> OcrResult<()> {
        let engine = OcrEngine::new_with_config_and_bytes(
            det_model_data,
            rec_model_data,
            keys_data,
            rect_border_size,
            merge_boxes,
            merge_threshold,
        )?;

        // 获取或初始化全局实例
        let instance = INSTANCE.get_or_init(|| Arc::new(Mutex::new(None)));

        // 更新引擎实例
        let mut guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        *guard = Some(engine);

        Ok(())
    }

    /// 获取全局OCR引擎实例
    ///
    /// Get the global OCR engine instance
    pub fn get_instance() -> OcrResult<Arc<Mutex<Option<OcrEngine>>>> {
        INSTANCE
            .get()
            .cloned()
            .ok_or_else(|| OcrError::EngineError("OCR engine not initialized".to_string()))
    }

    /// 在图像中检测文本区域
    ///
    /// Detect text regions in the image
    pub fn detect_text(image: DynamicImage) -> OcrResult<Vec<DynamicImage>> {
        let instance = Self::get_instance()?;
        let guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        let engine = guard
            .as_ref()
            .ok_or_else(|| OcrError::EngineError("OCR engine not initialized".to_string()))?;

        engine.detect_text(image)
    }

    /// 获取文本区域的矩形框
    ///
    /// Get text region rectangles
    pub fn get_text_rects(image: &DynamicImage) -> OcrResult<Vec<Rect>> {
        let instance = Self::get_instance()?;
        let guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        let engine = guard
            .as_ref()
            .ok_or_else(|| OcrError::EngineError("OCR engine not initialized".to_string()))?;

        engine.get_text_rects(image)
    }

    /// 获取文本区域图像
    ///
    /// Get text region images
    pub fn get_text_images(image: &DynamicImage) -> OcrResult<Vec<DynamicImage>> {
        let instance = Self::get_instance()?;
        let guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        let engine = guard
            .as_ref()
            .ok_or_else(|| OcrError::EngineError("OCR engine not initialized".to_string()))?;

        engine.get_text_images(image)
    }

    /// 识别图像中的文本
    ///
    /// Recognize text in the image
    pub fn recognize_text(image: DynamicImage) -> OcrResult<String> {
        let instance = Self::get_instance()?;
        let guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        let engine = guard
            .as_ref()
            .ok_or_else(|| OcrError::EngineError("OCR engine not initialized".to_string()))?;

        engine.recognize_text(image)
    }

    /// 完整的OCR处理，检测并识别图像中的所有文本
    ///
    /// Complete OCR processing, detecting and recognizing all text in the image
    pub fn process_ocr(image: DynamicImage) -> OcrResult<Vec<String>> {
        let instance = Self::get_instance()?;
        let guard = instance.lock().map_err(|_| {
            OcrError::EngineError("Failed to acquire lock on OCR engine manager".to_string())
        })?;

        let engine = guard
            .as_ref()
            .ok_or_else(|| OcrError::EngineError("OCR engine not initialized".to_string()))?;

        engine.process_ocr(image)
    }
}
