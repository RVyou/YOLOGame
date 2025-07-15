# Rust PaddleOCR

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

PaddleOCR 모델을 기반으로 Rust로 구현된 경량 및 효율적인 OCR(광학 문자 인식) 라이브러리입니다. 이 라이브러리는 MNN 추론 프레임워크를 활용하여 고성능 텍스트 감지 및 인식 기능을 제공하며, 완전한 C API 인터페이스를 제공합니다.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## 특징

- **텍스트 감지**: 이미지에서 텍스트 영역을 정확하게 찾기
- **텍스트 인식**: 감지된 영역에서 텍스트 내용 인식하기
- **다중 버전 모델 지원**: PP-OCRv4 및 PP-OCRv5 모델을 지원하여 유연하게 선택 사용
- **다국어 지원**: PP-OCRv5는 간체 중국어, 번체 중국어, 영어, 일본어, 중국어 병음 등 다양한 문자 유형 지원
- **복잡한 시나리오 인식**: 강화된 손글씨, 세로 텍스트, 생소한 문자 인식 능력
- **고성능**: MNN 추론 프레임워크로 최적화됨
- **최소한의 의존성**: 경량이며 쉽게 통합 가능
- **사용자 정의 가능**: 다양한 사용 사례에 맞게 조정 가능한 매개변수
- **명령줄 도구**: OCR 인식을 위한 간단한 명령줄 인터페이스
- **C API 지원**: 완전한 C 언어 인터페이스 제공으로 다양한 언어에서 호출 가능
- **메모리 안전성**: 자동 메모리 관리로 메모리 누수 방지

## 모델 버전

이 라이브러리는 두 가지 PaddleOCR 모델 버전을 지원합니다:

### PP-OCRv4
- **안정 버전**: 충분히 검증되었으며 호환성이 우수함
- **적용 시나리오**: 일반 문서 인식, 정확성이 높게 요구되는 시나리오
- **모델 파일**:
  - 감지 모델: `ch_PP-OCRv4_det_infer.mnn`
  - 인식 모델: `ch_PP-OCRv4_rec_infer.mnn`
  - 문자 집합: `ppocr_keys_v4.txt`

### PP-OCRv5 ⭐️ 권장
- **최신 버전**: 차세대 문자 인식 솔루션
- **다중 문자 유형 지원**: 간체 중국어, 중국어 병음, 번체 중국어, 영어, 일본어
- **강화된 시나리오 인식**:
  - 중영 복합 손글씨 인식 능력 현저히 향상
  - 세로 텍스트 인식 최적화
  - 생소한 문자 인식 능력 강화
- **성능 향상**: PP-OCRv4 대비 엔드 투 엔드 13% 향상
- **모델 파일**:
  - 감지 모델: `PP-OCRv5_mobile_det.mnn`
  - 인식 모델: `PP-OCRv5_mobile_rec.mnn`
  - 문자 집합: `ppocr_keys_v5.txt`

## 설치

`Cargo.toml`에 다음을 추가하세요:

```toml
[dependencies.rust-paddle-ocr]
git = "https://github.com/zibo-chen/rust-paddle-ocr.git"
```

특정 브랜치나 태그를 지정할 수도 있습니다:

```toml
[dependencies.rust-paddle-ocr]
git = "https://github.com/zibo-chen/rust-paddle-ocr.git"
branch = "main" 
```

### 전제 조건

이 라이브러리는 다음이 필요합니다:
- MNN 형식으로 변환된 사전 훈련된 PaddleOCR 모델
- 텍스트 인식을 위한 문자 집합 파일

## 사용 방법

### Rust 라이브러리로 사용

PP-OCRv4 또는 PP-OCRv5 모델을 유연하게 선택하여 사용할 수 있으며, 다른 모델 파일을 로드하기만 하면 됩니다:

```rust
use rust_paddle_ocr::{Det, Rec};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === PP-OCRv5 모델 사용 (권장) ===
    let mut det = Det::from_file("./models/PP-OCRv5_mobile_det.mnn")?;
    let mut rec = Rec::from_file(
        "./models/PP-OCRv5_mobile_rec.mnn", 
        "./models/ppocr_keys_v5.txt"
    )?;
    
    // === 또는 PP-OCRv4 모델 사용 ===
    // let mut det = Det::from_file("./models/ch_PP-OCRv4_det_infer.mnn")?;
    // let mut rec = Rec::from_file(
    //     "./models/ch_PP-OCRv4_rec_infer.mnn", 
    //     "./models/ppocr_keys_v4.txt"
    // )?;
    
    // 감지 매개변수 사용자 정의 (선택 사항)
    let det = det
        .with_rect_border_size(12)  // PP-OCRv5 권장 매개변수
        .with_merge_boxes(false)    // PP-OCRv5 권장 매개변수
        .with_merge_threshold(1);   // PP-OCRv5 권장 매개변수
    
    // 인식 매개변수 사용자 정의 (선택 사항)
    let rec = rec
        .with_min_score(0.6)
        .with_punct_min_score(0.1);
    
    // 이미지 열기
    let img = open("path/to/image.jpg")?;
    
    // 텍스트 영역 감지
    let text_images = det.find_text_img(&img)?;
    
    // 각 감지된 영역에서 텍스트 인식하기
    for text_img in text_images {
        let text = rec.predict_str(&text_img)?;
        println!("인식된 텍스트: {}", text);
    }
    
    Ok(())
}
```

### C 라이브러리로 사용

이 라이브러리는 완전한 C API 인터페이스를 제공하여 C/C++ 프로젝트에서 사용할 수 있습니다:

#### C 동적 라이브러리 컴파일

```bash
# 동적 라이브러리 컴파일
cargo build --release

# 생성된 동적 라이브러리 위치 (시스템에 따라 다름):
# Linux: target/release/librust_paddle_ocr.so
# macOS: target/release/librust_paddle_ocr.dylib  
# Windows: target/release/rust_paddle_ocr.dll

# C 헤더 파일이 프로젝트 루트 디렉토리에 자동 생성됩니다: rocr.h
```

#### C API 사용 예시

```c
#include "rocr.h"
#include <stdio.h>

int main() {
    // 버전 정보 가져오기
    printf("OCR 라이브러리 버전: %s\n", rocr_version());
    
    // OCR 엔진 생성
    ROCR_RocrHandle engine = rocr_create_engine(
        "./models/PP-OCRv5_mobile_det.mnn",
        "./models/PP-OCRv5_mobile_rec.mnn", 
        "./models/ppocr_keys_v5.txt"
    );
    
    if (engine == 0) {
        printf("OCR 엔진 생성 실패\n");
        return 1;
    }
    
    // 간단 모드 인식 - 텍스트 내용만 가져오기
    struct ROCR_RocrSimpleResult simple_result = 
        rocr_recognize_simple(engine, "./image.jpg");
    
    if (simple_result.STATUS == ROCR_RocrStatus_Success) {
        printf("%zu개의 텍스트 인식됨:\n", simple_result.COUNT);
        for (size_t i = 0; i < simple_result.COUNT; i++) {
            printf("- %s\n", simple_result.TEXTS[i]);
        }
    }
    
    // 간단 결과 메모리 해제
    rocr_free_simple_result(&simple_result);
    
    // 상세 모드 인식 - 텍스트와 위치 정보 가져오기
    struct ROCR_RocrResult detailed_result = 
        rocr_recognize_detailed(engine, "./image.jpg");
    
    if (detailed_result.STATUS == ROCR_RocrStatus_Success) {
        printf("%zu개의 텍스트 상자 상세 인식됨:\n", detailed_result.COUNT);
        for (size_t i = 0; i < detailed_result.COUNT; i++) {
            struct ROCR_RocrTextBox* box = &detailed_result.BOXES[i];
            printf("텍스트: %s\n", box->TEXT);
            printf("신뢰도: %.2f\n", box->CONFIDENCE);
            printf("위치: (%d, %d, %u, %u)\n", 
                   box->LEFT, box->TOP, box->WIDTH, box->HEIGHT);
        }
    }
    
    // 상세 결과 메모리 해제
    rocr_free_result(&detailed_result);
    
    // 엔진 소멸
    rocr_destroy_engine(engine);
    
    // 모든 리소스 정리
    rocr_cleanup();
    
    return 0;
}
```

#### C 데모 컴파일 및 실행

프로젝트에서 완전한 C 언어 데모 프로그램을 제공합니다:

```bash
# demo 디렉토리로 이동
cd demo

# C 데모 컴파일 (Linux/macOS)
gcc -o c_demo c_demo.c -L../target/release -lrust_paddle_ocr -ldl

# 데모 실행
./c_demo

# Windows 컴파일 예시
# gcc -o c_demo.exe c_demo.c -L../target/release -lrust_paddle_ocr -lws2_32 -luserenv
```

#### C API 고급 설정

```c
// 사용자 정의 설정으로 엔진 생성
ROCR_RocrHandle engine = rocr_create_engine_with_config(
    det_model_path,
    rec_model_path, 
    keys_path,
    12,    // rect_border_size - 경계 상자 확장 크기
    0,     // merge_boxes - 텍스트 상자 병합 여부 (0=false, 1=true)
    1      // merge_threshold - 병합 임계값
);

// 메모리 내 모델 데이터로 엔진 생성
ROCR_RocrHandle engine = rocr_create_engine_with_bytes(
    det_model_data, det_model_size,
    rec_model_data, rec_model_size,
    keys_data, keys_size,
    12, 0, 1
);
```

## 명령줄 도구

이 라이브러리는 직접 OCR 인식을 수행할 수 있는 내장 명령줄 도구를 제공합니다:

```bash
# 기본 사용법
./ocr -p path/to/image.jpg

# JSON 형식으로 출력 (자세한 정보와 위치 포함)
./ocr -p path/to/image.jpg -m json

# 상세 로그 정보 표시
./ocr -p path/to/image.jpg -v

# 현재 사용 중인 모델 버전 표시
./ocr --version-info
```

### 다른 버전 컴파일

```bash
# PP-OCRv4 모델 사용 버전 컴파일 (기본값)
cargo build --release

# PP-OCRv5 모델 사용 버전 컴파일 (권장)
cargo build --release --features v5
```

### 명령줄 옵션

```
옵션:
  -p, --path <IMAGE_PATH>  인식할 이미지 경로
  -m, --mode <MODE>        출력 모드: json(상세) 또는 text(간단) [기본값: text]
  -v, --verbose            상세 로그 정보 표시 여부
      --version-info       모델 버전 정보 표시
  -h, --help               도움말 정보 출력
  -V, --version            버전 정보 출력
```

## 모델 파일 획득

다음 경로에서 사전 훈련된 MNN 모델을 얻을 수 있습니다:

1. **공식 모델**: PaddleOCR 공식 저장소에서 다운로드하여 MNN 형식으로 변환
2. **프로젝트 제공**: 본 프로젝트의 `models/` 디렉토리에 변환된 모델 파일 포함

## PP-OCRv5 vs PP-OCRv4 성능 비교

| 특징 | PP-OCRv4 | PP-OCRv5 |
|------|----------|----------|
| 문자 유형 지원 | 중국어, 영어 | 간체 중국어, 번체 중국어, 영어, 일본어, 중국어 병음 |
| 손글씨 인식 | 기본 지원 | 현저히 향상 |
| 세로 텍스트 | 기본 지원 | 최적화 향상 |
| 생소한 문자 인식 | 제한적 지원 | 강화된 인식 |
| 엔드 투 엔드 정확도 | 기준 | 13% 향상 |
| 권장 시나리오 | 일반 문서 | 복잡하고 다양한 시나리오 |

## API 참조

### Rust API

#### 텍스트 감지 (Det)

```rust
// 새 감지기 생성
let mut det = Det::from_file("path/to/det_model.mnn")?;

// 텍스트 영역을 찾고 사각형 반환
let rects = det.find_text_rect(&img)?;

// 텍스트 영역을 찾고 자른 이미지 반환
let text_images = det.find_text_img(&img)?;

// 사용자 정의 옵션
let det = det
    .with_rect_border_size(12)      // 감지된 영역의 테두리 크기 설정
    .with_merge_boxes(false)        // 인접한 상자 병합 활성화/비활성화
    .with_merge_threshold(1);       // 상자 병합 임계값 설정
```

#### 텍스트 인식 (Rec)

```rust
// 새 인식기 생성
let mut rec = Rec::from_file("path/to/rec_model.mnn", "path/to/keys.txt")?;

// 텍스트 인식하고 문자열 반환
let text = rec.predict_str(&text_img)?;

// 텍스트 인식하고 신뢰도 점수와 함께 문자 반환
let char_scores = rec.predict_char_score(&text_img)?;

// 사용자 정의 옵션
let rec = rec
    .with_min_score(0.6)           // 일반 문자의 최소 신뢰도 설정
    .with_punct_min_score(0.1);    // 문장 부호의 최소 신뢰도 설정
```

### C API

#### 핵심 함수

```c
// 엔진 관리
ROCR_RocrHandle rocr_create_engine(const char* det_model, 
                                   const char* rec_model, 
                                   const char* keys_file);
ROCR_RocrHandle rocr_create_engine_with_config(...);
ROCR_RocrHandle rocr_create_engine_with_bytes(...);
enum ROCR_RocrStatus rocr_destroy_engine(ROCR_RocrHandle handle);

// 텍스트 인식
struct ROCR_RocrSimpleResult rocr_recognize_simple(ROCR_RocrHandle handle, 
                                                   const char* image_path);
struct ROCR_RocrResult rocr_recognize_detailed(ROCR_RocrHandle handle, 
                                               const char* image_path);

// 메모리 관리
void rocr_free_simple_result(struct ROCR_RocrSimpleResult* result);
void rocr_free_result(struct ROCR_RocrResult* result);
void rocr_cleanup(void);

// 유틸리티 함수
const char* rocr_version(void);
```

#### 데이터 구조

```c
// 상태 코드
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

// 텍스트 상자
typedef struct ROCR_RocrTextBox {
    char* TEXT;              // 인식된 텍스트
    float CONFIDENCE;        // 신뢰도 (0.0-1.0)
    int LEFT;               // 왼쪽 경계
    int TOP;                // 위쪽 경계  
    unsigned int WIDTH;     // 너비
    unsigned int HEIGHT;    // 높이
} ROCR_RocrTextBox;

// 상세 결과
typedef struct ROCR_RocrResult {
    enum ROCR_RocrStatus STATUS;     // 상태 코드
    size_t COUNT;                    // 텍스트 상자 수
    struct ROCR_RocrTextBox* BOXES;  // 텍스트 상자 배열
} ROCR_RocrResult;

// 간단 결과
typedef struct ROCR_RocrSimpleResult {
    enum ROCR_RocrStatus STATUS;     // 상태 코드
    size_t COUNT;                    // 텍스트 수
    char** TEXTS;                    // 텍스트 배열
} ROCR_RocrSimpleResult;
```

#### 메모리 관리 주의사항

1. **결과 해제**: 반드시 해당 해제 함수를 호출하여 결과 메모리를 해제해야 함
2. **엔진 소멸**: 사용 완료 후 반드시 엔진 인스턴스를 소멸시켜야 함
3. **전역 정리**: 프로그램 종료 전 `rocr_cleanup()` 호출하여 모든 리소스 정리
4. **스레드 안전성**: 엔진 인스턴스는 스레드 안전하지 않으므로 멀티스레드 사용 시 추가 동기화 필요

## 데모 프로그램

프로젝트의 `demo/` 디렉토리에서 완전한 데모 프로그램을 제공합니다:

- **C 데모** (`demo/c_demo.c`): 완전한 C 언어 호출 예시로 간단 모드와 상세 모드 사용법 시연
- **모델 파일**: `models/` 디렉토리에 예시 모델 파일 포함
- **테스트 이미지**: `res/` 디렉토리에 테스트 이미지 포함

데모 실행:
```bash
# demo 디렉토리로 이동하여 실행
cd demo && ./c_demo
```

## 라이선스

이 프로젝트는 Apache License 2.0에 따라 라이선스가 부여됩니다 - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 감사의 말

- [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) - 원본 OCR 모델 및 연구 제공
- [MNN](https://github.com/alibaba/MNN) - 효율적인 신경망 추론 프레임워크 제공
- [mnn-rs](https://github.com/aftershootco/mnn-rs) - Rust를 위한 MNN 바인딩 제공
