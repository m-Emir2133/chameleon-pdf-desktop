# Chameleon PDF Desktop

A fast, offline PDF toolbox for Windows. Drag files onto the window and they are converted or processed on your own machine, with no uploads and no account.

It is the desktop companion of the Android app *Chameleon PDF*, built with [Tauri 2](https://tauri.app) (Rust + WebView2). The whole app is a single ~30 MB executable. **The interface is currently in Turkish only**; translations are welcome.

## Features

**Convert**
- **Office → PDF**: PowerPoint, Word and Excel files (`pptx`, `ppt`, `docx`, `doc`, `xlsx`, `xls`, `odp`, `odt`, `ods`, `rtf`, `csv` …) are converted the moment you drop them, using the Microsoft Office installed on your PC. Output is identical to Office's own "Save as PDF".
- **Scan document**: turn phone photos of paper into a clean PDF. The four corners of the page are found automatically (with the [DocAligner](https://github.com/DocsaidLab/DocAligner) model, falling back to classic edge detection), the perspective is corrected and shadows are removed. Reorder pages by dragging, fix corners with draggable handles and a magnifier, rotate, and choose *Scanned* (clean grey), *Color* or *Original*.
- **Images → PDF**: JPEGs are embedded without recompression.
- **PDF → images**: PNG or JPG at screen, normal or print quality (uses the PDF engine built into Windows).

**PDF tools**
- Merge (drag to reorder), split every *N* pages, extract pages (`1-3, 5, 8-`, in any order), rotate, compress (downsamples and recompresses embedded images; text and vector graphics are untouched).

**Edit**
- **Edit text**: click any text on a page and type a replacement, or click empty space to add new text. Font size, weight, text colour and background colour are measured from the page. Turkish and other non-Latin-1 characters work.
- **Page numbers** (`1`, `1 / 10`, `Page 1`, three positions) and **watermark** (diagonal, semi-transparent). Rotated pages are handled.

**Security**
- Add an AES-256 password, or remove a password you know.

Every output is written next to the source file with a Turkish suffix (`name - birleştirilmiş.pdf` = merged, `name - tarama.pdf` = scan, …); existing files are never overwritten.

### Known limitations
- **Text editing is an overlay.** The original text is covered with a box in the background colour and the new text is drawn on top, so it looks and prints changed, but copying or searching the PDF can still return the old text. True in-place editing is not possible in general because PDFs usually embed only the glyphs they use.
- Office → PDF needs Microsoft Office. Everything else works without it.
- Rotated text runs cannot be edited.

## Download

Get the latest `ChameleonPDF-windows-x64.zip` from [Releases](../../releases), unzip it anywhere and run `ChameleonPDF.exe`. Keep the `modeller` folder next to the exe: it holds the document-corner model. Without it, scanning still works with classic edge detection.

Requirements: Windows 10 or 11 (x64) with the WebView2 runtime (preinstalled on Windows 11). Microsoft Office only for Office → PDF.

## Build from source

```powershell
# 1. Rust (stable) from https://rustup.rs
# 2. Optional, for the scan model (~83 MB):
powershell -ExecutionPolicy Bypass -File scripts\download-model.ps1
# 3. Build
cd src-tauri
cargo build --release
# -> target\release\chameleon-pdf.exe ; copy src-tauri\modeller next to it
```

The `ort` crate downloads a prebuilt ONNX Runtime during the first build and links it statically. There is no separate frontend build step; the UI in `ui/` is plain HTML/CSS/JS.

### Layout

| Path | What it does |
|---|---|
| `src-tauri/src/main.rs` | Tauri commands, output naming, Office conversion driver |
| `src-tauri/src/office.ps1` | Office COM automation (runs hidden; never closes Office windows you already had open) |
| `src-tauri/src/pdf.rs` | Merge, split, extract, rotate, images → PDF (lopdf) |
| `src-tauri/src/duzen.rs` | Overlays (page numbers, watermark, text edits), compression, encryption |
| `src-tauri/src/yazi.rs` | Subset-embeds a Windows font as a Type0/Identity-H font |
| `src-tauri/src/tarama.rs` | Corner detection (classic), perspective warp, shadow removal |
| `src-tauri/src/model.rs` | DocAligner corner model via ONNX Runtime |
| `src-tauri/src/gorsel.rs` | PDF → image via `Windows.Data.Pdf` |
| `ui/` | The interface (pdf.js is vendored in `ui/pdfjs/`) |

Identifiers are in Turkish (`birlestir` = merge, `bol` = split, `tarama` = scan, `yazi` = font …).

### Tests

```powershell
cd src-tauri
$env:CHAMELEON_TEST = "C:\path\to\folder\with\two\pdfs\and\some\jpgs"
$env:CHAMELEON_TARAMA = "C:\path\to\photos\with\gercek.json"   # optional, for corner-detection accuracy
cargo test -- --nocapture --test-threads=1
```

The tests run the real operations on your files and check the results by rendering them with the Windows PDF engine. `gercek.json` maps each photo name to its four ground-truth corners (`[[x,y],…]`, 0–1, clockwise from top-left).

On a synthetic set of 16 hard photos (cluttered desk, hard shadows, a hand over the page, low-contrast background, strong perspective), the classic detector found the corners within 1.5 % of the image diagonal in 5/16 photos; the DocAligner model did in 16/16, with corner errors of 4–10 px on 2000 px images and about 60 ms per photo on CPU.

## License

[MIT](LICENSE). Third-party components and their licenses are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
