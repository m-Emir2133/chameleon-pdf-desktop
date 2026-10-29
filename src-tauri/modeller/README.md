# Model folder

`docaligner-fastvit_sa24.onnx` is the [DocAligner](https://github.com/DocsaidLab/DocAligner) heatmap model
(`fastvit_sa24_h_e_bifpn_256_fp32.onnx`, ~83 MB) used by **Scan document** to find the page corners.
It is not stored in git or shipped with the app: choose it in the app (Scan document → Köşe bulucu → İndir), or run `scripts/download-model.ps1` for a development build.

- Input `img`: `[1, 3, 256, 256]` float32, **BGR**, divided by 255, image stretched to 256×256 (aspect ratio not kept).
- Output `heatmap`: `[1, 4, 128, 128]`, one channel per corner (top-left, top-right, bottom-right, bottom-left).
- Post-processing (`src/model.rs`): per channel, the centroid of the largest connected region above 0.3.

Code license: Apache-2.0. The DocAligner project does not state a separate license for the weights.
