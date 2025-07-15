# Rust PaddleOCR

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

PaddleOCRモデルをベースにしたRustで実装された軽量で効率的なOCR（光学文字認識）ライブラリです。このライブラリはMNN推論フレームワークを活用して、高性能なテキスト検出と認識機能を提供し、完全なC APIインターフェースを提供します。

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## 特徴

- **テキスト検出**: 画像内のテキスト領域を正確に特定
- **テキスト認識**: 検出された領域からテキスト内容を認識
- **複数モデルバージョン対応**: PP-OCRv4とPP-OCRv5モデルをサポート、柔軟に選択可能
- **多言語サポート**: PP-OCRv5は簡体中国語、繁体中国語、英語、日本語、中国語ピンインなど複数の文字タイプをサポート
- **複雑シーン認識**: 手書き文字、縦書きテキスト、難読漢字の認識能力を強化
- **高性能**: MNN推論フレームワークで最適化
- **最小限の依存関係**: 軽量で容易に統合可能
- **カスタマイズ可能**: 異なるユースケース向けの調整可能なパラメータ
- **コマンドラインツール**: OCR認識のための簡単なコマンドラインインターフェース
- **C API サポート**: 完全なC言語インターフェースを提供、クロスランゲージ呼び出しをサポート
- **メモリ安全**: 自動メモリ管理でメモリリークを防止

## モデルバージョン

このライブラリは2つのPaddleOCRモデルバージョンをサポートしています：

### PP-OCRv4
- **安定版**: 十分に検証済み、互換性が良い
- **適用シーン**: 一般的な文書認識、精度要求の高いシーン
- **モデルファイル**:
  - 検出モデル: `ch_PP-OCRv4_det_infer.mnn`
  - 認識モデル: `ch_PP-OCRv4_rec_infer.mnn`
  - 文字セット: `ppocr_keys_v4.txt`

### PP-OCRv5 ⭐️ 推奨
- **最新版**: 新世代文字認識ソリューション
- **多文字タイプサポート**: 簡体中国語、中国語ピンイン、繁体中国語、英語、日本語
- **シーン認識の強化**:
  - 中英複雑手書き文字認識能力が大幅向上
  - 縦書きテキスト認識の最適化
  - 難読漢字認識能力の強化
- **性能向上**: PP-OCRv4と比較してエンドツーエンドで13ポイント向上
- **モデルファイル**:
  - 検出モデル: `PP-OCRv5_mobile_det.mnn`
  - 認識モデル: `PP-OCRv5_mobile_rec.mnn`
  - 文字セット: `ppocr_keys_v5.txt`

## インストール

`Cargo.toml`に以下を追加してください：

```toml
[dependencies.rust-paddle-ocr]
git = "https://github.com/zibo-chen/rust-paddle-ocr.git"
```

特定のブランチやタグを指定することもできます：

```toml
[dependencies.rust-paddle-ocr]
git = "https://github.com/zibo-chen/rust-paddle-ocr.git"
branch = "main" 
```

### 前提条件

このライブラリには以下が必要です：
- MNN形式に変換された事前学習済みPaddleOCRモデル
- テキスト認識用の文字セットファイル

## 使用方法

### Rustライブラリとして使用

PP-OCRv4またはPP-OCRv5モデルを柔軟に選択でき、異なるモデルファイルを読み込むだけで切り替え可能です：

```rust
use rust_paddle_ocr::{Det, Rec};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === PP-OCRv5モデルを使用（推奨） ===
    let mut det = Det::from_file("./models/PP-OCRv5_mobile_det.mnn")?;
    let mut rec = Rec::from_file(
        "./models/PP-OCRv5_mobile_rec.mnn", 
        "./models/ppocr_keys_v5.txt"
    )?;
    
    // === またはPP-OCRv4モデルを使用 ===
    // let mut det = Det::from_file("./models/ch_PP-OCRv4_det_infer.mnn")?;
    // let mut rec = Rec::from_file(
    //     "./models/ch_PP-OCRv4_rec_infer.mnn", 
    //     "./models/ppocr_keys_v4.txt"
    // )?;
    
    // 検出パラメータをカスタマイズ（任意）
    let det = det
        .with_rect_border_size(12)  // PP-OCRv5推奨パラメータ
        .with_merge_boxes(false)    // PP-OCRv5推奨パラメータ
        .with_merge_threshold(1);   // PP-OCRv5推奨パラメータ
    
    // 認識パラメータをカスタマイズ（任意）
    let rec = rec
        .with_min_score(0.6)
        .with_punct_min_score(0.1);
    
    // 画像を開く
    let img = open("path/to/image.jpg")?;
    
    // テキスト領域を検出
    let text_images = det.find_text_img(&img)?;
    
    // 検出された各領域のテキストを認識
    for text_img in text_images {
        let text = rec.predict_str(&text_img)?;
        println!("認識されたテキスト: {}", text);
    }
    
    Ok(())
}
```

### C ライブラリとして使用

このライブラリは完全なC APIインターフェースを提供し、C/C++プロジェクトで使用できます：

#### C動的ライブラリのコンパイル

```bash
# 動的ライブラリのコンパイル
cargo build --release

# 生成される動的ライブラリの場所（システムによって異なる）：
# Linux: target/release/librust_paddle_ocr.so
# macOS: target/release/librust_paddle_ocr.dylib  
# Windows: target/release/rust_paddle_ocr.dll

# Cヘッダーファイルはプロジェクトルートに自動生成されます：rocr.h
```

#### C API 使用例

```c
#include "rocr.h"
#include <stdio.h>

int main() {
    // バージョン情報の取得
    printf("OCRライブラリバージョン: %s\n", rocr_version());
    
    // OCRエンジンの作成
    ROCR_RocrHandle engine = rocr_create_engine(
        "./models/PP-OCRv5_mobile_det.mnn",
        "./models/PP-OCRv5_mobile_rec.mnn", 
        "./models/ppocr_keys_v5.txt"
    );
    
    if (engine == 0) {
        printf("OCRエンジンの作成に失敗しました\n");
        return 1;
    }
    
    // シンプルモード認識 - テキスト内容のみ取得
    struct ROCR_RocrSimpleResult simple_result = 
        rocr_recognize_simple(engine, "./image.jpg");
    
    if (simple_result.STATUS == ROCR_RocrStatus_Success) {
        printf("%zu個のテキストを認識しました:\n", simple_result.COUNT);
        for (size_t i = 0; i < simple_result.COUNT; i++) {
            printf("- %s\n", simple_result.TEXTS[i]);
        }
    }
    
    // シンプル結果のメモリ解放
    rocr_free_simple_result(&simple_result);
    
    // 詳細モード認識 - テキストと位置情報を取得
    struct ROCR_RocrResult detailed_result = 
        rocr_recognize_detailed(engine, "./image.jpg");
    
    if (detailed_result.STATUS == ROCR_RocrStatus_Success) {
        printf("%zu個のテキストボックスを詳細認識しました:\n", detailed_result.COUNT);
        for (size_t i = 0; i < detailed_result.COUNT; i++) {
            struct ROCR_RocrTextBox* box = &detailed_result.BOXES[i];
            printf("テキスト: %s\n", box->TEXT);
            printf("信頼度: %.2f\n", box->CONFIDENCE);
            printf("位置: (%d, %d, %u, %u)\n", 
                   box->LEFT, box->TOP, box->WIDTH, box->HEIGHT);
        }
    }
    
    // 詳細結果のメモリ解放
    rocr_free_result(&detailed_result);
    
    // エンジンの破棄
    rocr_destroy_engine(engine);
    
    // すべてのリソースのクリーンアップ
    rocr_cleanup();
    
    return 0;
}
```

#### C Demoのコンパイルと実行

プロジェクトは完全なC言語デモプログラムを提供しています：

```bash
# demoディレクトリに移動
cd demo

# C demoのコンパイル（Linux/macOS）
gcc -o c_demo c_demo.c -L../target/release -lrust_paddle_ocr -ldl

# demoの実行
./c_demo

# Windowsコンパイル例
# gcc -o c_demo.exe c_demo.c -L../target/release -lrust_paddle_ocr -lws2_32 -luserenv
```

#### C API 高度な設定

```c
// カスタム設定でエンジンを作成
ROCR_RocrHandle engine = rocr_create_engine_with_config(
    det_model_path,
    rec_model_path, 
    keys_path,
    12,    // rect_border_size - 境界ボックス拡張サイズ
    0,     // merge_boxes - テキストボックスを結合するか (0=false, 1=true)
    1      // merge_threshold - 結合閾値
);

// メモリ内のモデルデータでエンジンを作成
ROCR_RocrHandle engine = rocr_create_engine_with_bytes(
    det_model_data, det_model_size,
    rec_model_data, rec_model_size,
    keys_data, keys_size,
    12, 0, 1
);
```

## コマンドラインツール

このライブラリには、直接OCR認識を行うための組み込みコマンドラインツールが提供されています：

```bash
# 基本的な使用法
./ocr -p path/to/image.jpg

# JSON形式で出力（詳細情報と位置を含む）
./ocr -p path/to/image.jpg -m json

# 詳細なログ情報を表示
./ocr -p path/to/image.jpg -v

# 現在使用中のモデルバージョンを表示
./ocr --version-info
```

### 異なるバージョンのコンパイル

```bash
# PP-OCRv4モデルを使用するバージョンをコンパイル（デフォルト）
cargo build --release

# PP-OCRv5モデルを使用するバージョンをコンパイル（推奨）
cargo build --release --features v5
```

### コマンドラインオプション

```
オプション:
  -p, --path <IMAGE_PATH>  認識する画像のパス
  -m, --mode <MODE>        出力モード: json(詳細) または text(シンプル) [デフォルト: text]
  -v, --verbose            詳細なログ情報を表示するかどうか
      --version-info       モデルバージョン情報を表示
  -h, --help               ヘルプ情報を表示
  -V, --version            バージョン情報を表示
```

## モデルファイルの取得

以下のチャンネルから事前学習済みMNNモデルを取得できます：

1. **公式モデル**: PaddleOCR公式リポジトリからダウンロードしてMNN形式に変換
2. **プロジェクト提供**: 本プロジェクトの`models/`ディレクトリに変換済みモデルファイルを含む

## PP-OCRv5 vs PP-OCRv4 性能比較

| 特徴 | PP-OCRv4 | PP-OCRv5 |
|------|----------|----------|
| 文字タイプサポート | 中国語、英語 | 簡体中国語、繁体中国語、英語、日本語、中国語ピンイン |
| 手書き文字認識 | 基本サポート | 大幅強化 |
| 縦書きテキスト | 基本サポート | 最適化向上 |
| 難読漢字認識 | 限定サポート | 認識強化 |
| エンドツーエンド精度 | ベースライン | 13%向上 |
| 推奨シーン | 一般文書 | 複雑多様シーン |

## API リファレンス

### Rust API

#### テキスト検出 (Det)

```rust
// 新しい検出器を作成
let mut det = Det::from_file("path/to/det_model.mnn")?;

// テキスト領域を検出して矩形を返す
let rects = det.find_text_rect(&img)?;

// テキスト領域を検出して切り取られた画像を返す
let text_images = det.find_text_img(&img)?;

// カスタマイズオプション
let det = det
    .with_rect_border_size(12)
    .with_merge_boxes(false)
    .with_merge_threshold(1);
```

#### テキスト認識 (Rec)

```rust
// 新しい認識器を作成
let mut rec = Rec::from_file("path/to/rec_model.mnn", "path/to/keys.txt")?;

// テキストを認識して文字列を返す
let text = rec.predict_str(&text_img)?;

// テキストを認識して信頼度スコア付きの文字を返す
let char_scores = rec.predict_char_score(&text_img)?;

// カスタマイズオプション
let rec = rec
    .with_min_score(0.6)           // 通常文字の最小信頼度を設定
    .with_punct_min_score(0.1);    // 句読点の最小信頼度を設定
```

### C API

#### コア関数

```c
// エンジン管理
ROCR_RocrHandle rocr_create_engine(const char* det_model, 
                                   const char* rec_model, 
                                   const char* keys_file);
ROCR_RocrHandle rocr_create_engine_with_config(...);
ROCR_RocrHandle rocr_create_engine_with_bytes(...);
enum ROCR_RocrStatus rocr_destroy_engine(ROCR_RocrHandle handle);

// テキスト認識
struct ROCR_RocrSimpleResult rocr_recognize_simple(ROCR_RocrHandle handle, 
                                                   const char* image_path);
struct ROCR_RocrResult rocr_recognize_detailed(ROCR_RocrHandle handle, 
                                               const char* image_path);

// メモリ管理
void rocr_free_simple_result(struct ROCR_RocrSimpleResult* result);
void rocr_free_result(struct ROCR_RocrResult* result);
void rocr_cleanup(void);

// ユーティリティ関数
const char* rocr_version(void);
```

#### データ構造

```c
// ステータスコード
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

// テキストボックス
typedef struct ROCR_RocrTextBox {
    char* TEXT;              // 認識されたテキスト
    float CONFIDENCE;        // 信頼度 (0.0-1.0)
    int LEFT;               // 左境界
    int TOP;                // 上境界  
    unsigned int WIDTH;     // 幅
    unsigned int HEIGHT;    // 高さ
} ROCR_RocrTextBox;

// 詳細結果
typedef struct ROCR_RocrResult {
    enum ROCR_RocrStatus STATUS;     // ステータスコード
    size_t COUNT;                    // テキストボックス数
    struct ROCR_RocrTextBox* BOXES;  // テキストボックス配列
} ROCR_RocrResult;

// シンプル結果
typedef struct ROCR_RocrSimpleResult {
    enum ROCR_RocrStatus STATUS;     // ステータスコード
    size_t COUNT;                    // テキスト数
    char** TEXTS;                    // テキスト配列
} ROCR_RocrSimpleResult;
```

#### メモリ管理の注意事項

1. **結果の解放**: 対応する解放関数を呼び出して結果のメモリを解放する必要があります
2. **エンジンの破棄**: 使用後は必ずエンジンインスタンスを破棄してください
3. **グローバルクリーンアップ**: プログラム終了前に `rocr_cleanup()` を呼び出してすべてのリソースをクリーンアップしてください
4. **スレッドセーフティ**: エンジンインスタンスはスレッドセーフではありません。マルチスレッドで使用する場合は追加の同期が必要です

## デモプログラム

プロジェクトは `demo/` ディレクトリに完全なデモプログラムを提供しています：

- **C Demo** (`demo/c_demo.c`)：完全なC言語呼び出し例、シンプルモードと詳細モードの使用方法を示します
- **モデルファイル**：`models/` ディレクトリにサンプルモデルファイルが含まれています
- **テスト画像**：`res/` ディレクトリにテスト画像が含まれています

デモの実行：
```bash
# demoディレクトリに移動して実行
cd demo && ./c_demo
```

## ライセンス

このプロジェクトはApache License 2.0でライセンスされています - 詳細は[LICENSE](LICENSE)ファイルをご覧ください。

## 謝辞

- [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) - 元のOCRモデルと研究のために
- [MNN](https://github.com/alibaba/MNN) - 効率的なニューラルネットワーク推論フレームワークのために
- [mnn-rs](https://github.com/aftershootco/mnn-rs) - RustにMNNバインディングを提供するために
