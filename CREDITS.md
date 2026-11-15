# Credits

## Rebuilt from

**AR Drawing: Sketch & Paint 5.2.8** — Android, package
`ar.drawing.sketch.paint.trace.draw.picture.paper`.

This project is an independent browser rebuild. Its screens, wording and tool
layout were recreated from the original app's own instruction videos, UI strings
and asset list. It carries none of the original's artwork, code or picture
library, and it is not affiliated with or endorsed by the original's publisher.

## Third-party files carried over

| File in this repo | Original path | Licence |
| --- | --- | --- |
| `assets/fonts/Poppins-*.ttf` | `assets/fonts/poppins/` | SIL Open Font License 1.1 |

Font licence: `assets/fonts/poppins/OFL.txt` in the app package (SIL Open Font
License 1.1). The Poppins typeface may be redistributed and embedded under that
licence.

## Recreated, not copied

- App mark, icons, banners and promo art are drawn from simple shapes and
  gradients; no logo or photographic asset from the APK is included.
- The ten line drawings in `src/art.rs` are new polylines, not the app's picture
  library.
- Every screen was re-implemented in Rust; none of the original's Dart code is
  used.

## Libraries

- `macroquad` / `miniquad` (MIT / Apache-2.0) — the renderer, input handling, and
  the `mq_js_bundle.js` loader used to run the WebAssembly build in a browser.
- `windows-sys` (MIT / Apache-2.0) — used only to park the screenshot window
  behind other windows on Windows.
