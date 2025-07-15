# Rust PaddleOCR

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

A lightweight and efficient OCR (Optical Character Recognition) library implemented in Rust, based on the PaddleOCR models. This library leverages the MNN inference framework to provide high-performance text detection and recognition capabilities, with complete C API support.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## Features

- **Text Detection**: Accurately locate text regions in images
- **Text Recognition**: Recognize text content from the detected regions
- **Multi-Version Model Support**: Support for both PP-OCRv4 and PP-OCRv5 models with flexible selection
- **Multi-Language Support**: PP-OCRv5 supports Simplified Chinese, Traditional Chinese, English, Japanese, Chinese Pinyin and more
- **Complex Scene Recognition**: Enhanced handwriting, vertical text, and rare character recognition capabilities
- **High Performance**: Optimized with the MNN inference framework
- **Minimal Dependencies**: Lightweight and easy to integrate
- **Customizable**: Adjustable parameters for different use cases
- **Command-line Tool**: Simple command-line interface for OCR recognition
- **C API Support**: Complete C language interface for cross-language integration
- **Memory Safety**: Automatic memory management with leak prevention

## Model Versions

This library supports two PaddleOCR model versions:

### PP-OCRv4
- **Stable Version**: Well-tested and highly compatible
- **Use Cases**: Regular document recognition, scenarios requiring high accuracy
- **Model Files**:
  - Detection model: `ch_PP-OCRv4_det_infer.mnn`
  - Recognition model: `ch_PP-OCRv4_rec_infer.mnn`
  - Character set: `ppocr_keys_v4.txt`

### PP-OCRv5 ⭐️ Recommended
- **Latest Version**: Next-generation text recognition solution
- **Multi-Script Support**: Simplified Chinese, Chinese Pinyin, Traditional Chinese, English, Japanese
- **Enhanced Scene Recognition**:
  - Significantly improved Chinese-English complex handwriting recognition
  - Optimized vertical text recognition
  - Enhanced rare character recognition capabilities
- **Performance Improvement**: 13% end-to-end improvement compared to PP-OCRv4
- **Model Files**:
  - Detection model: `PP-OCRv5_mobile_det.mnn`
  - Recognition model: `PP-OCRv5_mobile_rec.mnn`
  - Character set: `ppocr_keys_v5.txt`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies.rust-paddle-ocr]
git = "https://github.com/zibo-chen/rust-paddle-ocr.git"
```

You can also specify a particular branch or tag:

```toml
[dependencies.rust-paddle-ocr]
git = "https://github.com/zibo-chen/rust-paddle-ocr.git"
branch = "main"
```

### Prerequisites

This library requires:
- Pre-trained PaddleOCR models converted to MNN format
- Character set file for text recognition

## Usage

### As a Rust Library

You can flexibly choose between PP-OCRv4 or PP-OCRv5 models by simply loading different model files:

```rust
use rust_paddle_ocr::{Det, Rec};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === Using PP-OCRv5 models (Recommended) ===
    let mut det = Det::from_file("./models/PP-OCRv5_mobile_det.mnn")?;
    let mut rec = Rec::from_file(
        "./models/PP-OCRv5_mobile_rec.mnn", 
        "./models/ppocr_keys_v5.txt"
    )?;
    
    // === Or using PP-OCRv4 models ===
    // let mut det = Det::from_file("./models/ch_PP-OCRv4_det_infer.mnn")?;
    // let mut rec = Rec::from_file(
    //     "./models/ch_PP-OCRv4_rec_infer.mnn", 
    //     "./models/ppocr_keys_v4.txt"
    // )?;
    
    // Customize detection parameters (optional)
    let det = det
        .with_rect_border_size(12)  // Recommended for PP-OCRv5
        .with_merge_boxes(false)    // Recommended for PP-OCRv5
        .with_merge_threshold(1);   // Recommended for PP-OCRv5
    
    // Customize recognition parameters (optional)
    let rec = rec
        .with_min_score(0.6)
        .with_punct_min_score(0.1);
    
    // Open an image
    let img = open("path/to/image.jpg")?;
    
    // Detect text regions
    let text_images = det.find_text_img(&img)?;
    
    // Recognize text in each detected region
    for text_img in text_images {
        let text = rec.predict_str(&text_img)?;
        println!("Recognized text: {}", text);
    }
    
    Ok(())
}
```

### As a C Library

This library provides a complete C API interface for use in C/C++ projects:

#### Building the C Dynamic Library

```bash
# Build the dynamic library
cargo build --release

# Generated library location (depends on system):
# Linux: target/release/librust_paddle_ocr.so
# macOS: target/release/librust_paddle_ocr.dylib  
# Windows: target/release/rust_paddle_ocr.dll

# C header file is automatically generated in project root: rocr.h
```

#### C API Usage Example

```c
#include "rocr.h"
#include <stdio.h>

int main() {
    // Get version information
    printf("OCR Library Version: %s\n", rocr_version());
    
    // Create OCR engine
    ROCR_RocrHandle engine = rocr_create_engine(
        "./models/PP-OCRv5_mobile_det.mnn",
        "./models/PP-OCRv5_mobile_rec.mnn", 
        "./models/ppocr_keys_v5.txt"
    );
    
    if (engine == 0) {
        printf("Failed to create OCR engine\n");
        return 1;
    }
    
    // Simple mode recognition - get only text content
    struct ROCR_RocrSimpleResult simple_result = 
        rocr_recognize_simple(engine, "./image.jpg");
    
    if (simple_result.STATUS == ROCR_RocrStatus_Success) {
        printf("Recognized %zu texts:\n", simple_result.COUNT);
        for (size_t i = 0; i < simple_result.COUNT; i++) {
            printf("- %s\n", simple_result.TEXTS[i]);
        }
    }
    
    // Free simple result memory
    rocr_free_simple_result(&simple_result);
    
    // Detailed mode recognition - get text and position information
    struct ROCR_RocrResult detailed_result = 
        rocr_recognize_detailed(engine, "./image.jpg");
    
    if (detailed_result.STATUS == ROCR_RocrStatus_Success) {
        printf("Detailed recognition found %zu text boxes:\n", detailed_result.COUNT);
        for (size_t i = 0; i < detailed_result.COUNT; i++) {
            struct ROCR_RocrTextBox* box = &detailed_result.BOXES[i];
            printf("Text: %s\n", box->TEXT);
            printf("Confidence: %.2f\n", box->CONFIDENCE);
            printf("Position: (%d, %d, %u, %u)\n", 
                   box->LEFT, box->TOP, box->WIDTH, box->HEIGHT);
        }
    }
    
    // Free detailed result memory
    rocr_free_result(&detailed_result);
    
    // Destroy engine
    rocr_destroy_engine(engine);
    
    // Cleanup all resources
    rocr_cleanup();
    
    return 0;
}
```

#### Building and Running C Demo

The project provides a complete C language demonstration program:

```bash
# Enter demo directory
cd demo

# Compile C demo (Linux/macOS)
gcc -o c_demo c_demo.c -L../target/release -lrust_paddle_ocr -ldl

# Run demo
./c_demo

# Windows compilation example
# gcc -o c_demo.exe c_demo.c -L../target/release -lrust_paddle_ocr -lws2_32 -luserenv
```

#### Advanced C API Configuration

```c
// Create engine with custom configuration
ROCR_RocrHandle engine = rocr_create_engine_with_config(
    det_model_path,
    rec_model_path, 
    keys_path,
    12,    // rect_border_size - border expansion size
    0,     // merge_boxes - whether to merge text boxes (0=false, 1=true)
    1      // merge_threshold - merge threshold
);

// Create engine with model data in memory
ROCR_RocrHandle engine = rocr_create_engine_with_bytes(
    det_model_data, det_model_size,
    rec_model_data, rec_model_size,
    keys_data, keys_size,
    12, 0, 1
);
```

## API Reference

### Rust API

### Text Detection (Det)

```rust
// Create a new detector
let mut det = Det::from_file("path/to/det_model.mnn")?;

// Find text regions and return rectangles
let rects = det.find_text_rect(&img)?;

// Find text regions and return cropped images
let text_images = det.find_text_img(&img)?;

// Customization options
let det = det
    .with_rect_border_size(12)
    .with_merge_boxes(false)
    .with_merge_threshold(1);
```

### Text Recognition (Rec)

```rust
// Create a new recognizer
let mut rec = Rec::from_file("path/to/rec_model.mnn", "path/to/keys.txt")?;

// Recognize text and return a string
let text = rec.predict_str(&text_img)?;

// Recognize text and return characters with confidence scores
let char_scores = rec.predict_char_score(&text_img)?;

// Customization options
let rec = rec
    .with_min_score(0.6)           // Set minimum confidence for regular characters
    .with_punct_min_score(0.1);    // Set minimum confidence for punctuation
```

### C API

#### Core Functions

```c
// Engine management
ROCR_RocrHandle rocr_create_engine(const char* det_model, 
                                   const char* rec_model, 
                                   const char* keys_file);
ROCR_RocrHandle rocr_create_engine_with_config(...);
ROCR_RocrHandle rocr_create_engine_with_bytes(...);
enum ROCR_RocrStatus rocr_destroy_engine(ROCR_RocrHandle handle);

// Text recognition
struct ROCR_RocrSimpleResult rocr_recognize_simple(ROCR_RocrHandle handle, 
                                                   const char* image_path);
struct ROCR_RocrResult rocr_recognize_detailed(ROCR_RocrHandle handle, 
                                               const char* image_path);

// Memory management
void rocr_free_simple_result(struct ROCR_RocrSimpleResult* result);
void rocr_free_result(struct ROCR_RocrResult* result);
void rocr_cleanup(void);

// Utility functions
const char* rocr_version(void);
```

#### Data Structures

```c
// Status codes
typedef enum ROCR_RocrStatus {
    ROCR_RocrStatus_Success = 0,
    ROCR_RocrStatus_InitError = 1,
    ROCR_RocrStatus_FileNotFound = 2,
    ROCR_RocrStatus_ImageLoadError = 3,
    ROCR_RocrStatus_ProcessError = 4,
    ROCR_RocrStatus_MemoryError = 5,
    ROCR_RocrStatus_InvalidParam = 6,
    ROCR_RocrStatus_NotInitialized = 7,
} ROCR_RocrStatus;

// Text box
typedef struct ROCR_RocrTextBox {
    char* TEXT;              // Recognized text
    float CONFIDENCE;        // Confidence score (0.0-1.0)
    int LEFT;               // Left boundary
    int TOP;                // Top boundary  
    unsigned int WIDTH;     // Width
    unsigned int HEIGHT;    // Height
} ROCR_RocrTextBox;

// Detailed result
typedef struct ROCR_RocrResult {
    enum ROCR_RocrStatus STATUS;     // Status code
    size_t COUNT;                    // Number of text boxes
    struct ROCR_RocrTextBox* BOXES;  // Text box array
} ROCR_RocrResult;

// Simple result
typedef struct ROCR_RocrSimpleResult {
    enum ROCR_RocrStatus STATUS;     // Status code
    size_t COUNT;                    // Number of texts
    char** TEXTS;                    // Text array
} ROCR_RocrSimpleResult;
```

#### Memory Management Notes

1. **Result Cleanup**: Must call corresponding cleanup functions to free result memory
2. **Engine Destruction**: Must destroy engine instances when finished
3. **Global Cleanup**: Call `rocr_cleanup()` before program termination to clean all resources
4. **Thread Safety**: Engine instances are not thread-safe, requires additional synchronization for multi-threading

## Performance Optimization

This library includes several optimizations:
- Efficient tensor management
- Smart box merging for text detection
- Adaptive image preprocessing
- Configurable confidence thresholds

## Demo Programs

The project provides complete demonstration programs in the `demo/` directory:

- **C Demo** (`demo/c_demo.c`): Complete C language usage example, demonstrating both simple and detailed modes
- **Model Files**: The `models/` directory contains example model files
- **Test Images**: The `res/` directory contains test images

Run the demo:
```bash
# Enter demo directory and run
cd demo && ./c_demo
```

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) - For the original OCR models and research
- [MNN](https://github.com/alibaba/MNN) - For the efficient neural network inference framework
- [mnn-rs](https://github.com/aftershootco/mnn-rs) - For providing Rust bindings to MNN
