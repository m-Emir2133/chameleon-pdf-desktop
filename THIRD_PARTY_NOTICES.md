# Third-party notices

Chameleon PDF Desktop is MIT-licensed. It builds on the following projects, each under its own license.

## Bundled with the app

| Component | Use | License |
|---|---|---|
| [pdf.js](https://github.com/mozilla/pdf.js) (`ui/pdfjs/`) | Page rendering and text positions in the text editor | Apache-2.0 (`ui/pdfjs/LICENSE.txt`) |
| [ONNX Runtime](https://github.com/microsoft/onnxruntime) (linked statically through [`ort`](https://github.com/pykeio/ort)) | Runs the document-corner model | MIT |

## Rust crates (compiled in)

[Tauri](https://tauri.app) (MIT/Apache-2.0), [lopdf](https://github.com/J-F-Liu/lopdf) (MIT), [image](https://github.com/image-rs/image) and [imageproc](https://github.com/image-rs/imageproc) (MIT/Apache-2.0), [subsetter](https://github.com/typst/subsetter) and [ttf-parser](https://github.com/harfbuzz/ttf-parser) (MIT/Apache-2.0), [windows-rs](https://github.com/microsoft/windows-rs) (MIT/Apache-2.0), [rayon](https://github.com/rayon-rs/rayon) (MIT/Apache-2.0), [serde](https://serde.rs) (MIT/Apache-2.0), [base64](https://github.com/marshallpierce/rust-base64) (MIT/Apache-2.0), [getrandom](https://github.com/rust-random/getrandom) (MIT/Apache-2.0), [ureq](https://github.com/algesten/ureq) (MIT/Apache-2.0), [sha2](https://github.com/RustCrypto/hashes) (MIT/Apache-2.0), plus their dependencies. Run `cargo tree` in `src-tauri` for the full list.

## Not bundled

- **[DocAligner](https://github.com/DocsaidLab/DocAligner) models** (`fastvit_sa24`, `fastvit_t8`, `lcnet100`) are not distributed with this app. If the user chooses one, it is downloaded from the DocAligner project's own Google Drive files and checked against a fixed SHA-256. DocAligner's code is Apache-2.0; the project does not state a separate license for the weights, so their terms are between the user and the DocAligner authors.
- **Microsoft Office** (PowerPoint, Word, Excel) is used through COM automation when installed on the user's machine, for Office → PDF conversion.
- **Arial** (or Segoe UI) is read from the user's Windows fonts folder; only the glyphs actually used are subset-embedded into PDFs the user creates.
- **Windows.Data.Pdf** (part of Windows) renders PDF pages to images.
