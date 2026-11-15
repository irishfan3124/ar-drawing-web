# How the rebuild plays

This is a tracing app, not a shooter, so the numbers below are the ones that
shape the tracing loop: the overlay sheet, its opacity and scale, the lock and
flashlight, and the finger-drawing tray.

The screen design is a 430 × 900 phone canvas, scaled to fit the window.

## Screen orientation

The renderer on this PC hands back its framebuffer bottom-up, so the camera is
mirrored once (`zoom.y = -zoom.y`) on every platform. That single rule makes the
desktop window, the `--screenshot` PNG and the browser page all read top-down.

| Value | Value | Status |
| --- | --- | --- |
| Camera y origin | mirrored once on desktop and wasm | measured — a real window capture was upside down before the fix, and a headless Chrome capture of the web page was inverted without the mirror |
| Pointer y (desktop) | straight top-down screen space, no mirror | measured — clicking the visible "Step by step" card opened it |
| Pointer y (browser) | mirrored to match the canvas | measured — a tap on "Start drawing" at (112, 293) opened the sketch library |
| Screenshot rows | readback stored as-is; `Image::export_png` flips once | measured — `--screenshot` and the on-screen window now show the same top-down picture |

## What was read out of the original program

The Android app's own code is `lib/app/libapp.so` (Flutter ships its Dart code as
a native library). It was opened in Deep mode and read with the program tools;
nothing was executed.

| Fact | Value |
| --- | --- |
| File | `checks/apk/lib/libapp.so`, 30,573,128 bytes |
| SHA-256 | `f5a7a8bda267b666168ad439aebba7cd26a3d20a4e32ba62983e8d0a0d82feec` |
| Format | ELF 32-bit, ARM, shared library, **stripped** |
| Build id | `68aaeaa41ff6dada667cc342e4b66257` |
| Flutter / Dart | Flutter with Dart 3.13.0, snapshot hash `0451907c2eaa8467e848c0067bfe8ed4` |
| Snapshot layout | one unified snapshot: `_kDartSnapshotData` at `0x200`, `_kDartSnapshotText` at `0xa34000`, `_kDartSnapshotBuildId` at `0x114` |
| Sections | `.rodata` 10,689,816 bytes at `0x200`; `.text` 19,857,760 bytes at `0xa34000` |
| Analysed code | 45,134 functions, 3,579,111 instructions, 453,233 references, 111,316 strings |
| Recovered names | 12,208 named classes, 47,460 members, 86,918 strings, 52,541 code ranges |
| Known blind spot | 4,055 indirect jumps could not be resolved in the native pass, so a few call targets stay unknown |

The three exported names are the whole `.dynsym`: this is a Dart **AOT
snapshot**, not ordinary compiled code. Function names, types and variable names
do not survive, but the snapshot's class table, method table and string literals
do. That is what the rebuild uses.

### How a string was followed to a function

Every string in the snapshot is a heap object, so the native string search finds
no direct code reference: for example all 24 occurrences of "Flashlight" report
`references: []`. Dart code fetches constants through the object pool
(`[r5, #…]` in this ARM build), not through a string address. The entry prologue
of `sub_a34060` at `0xa34060` shows the thread register in use:

```
0xa3407c  ldr ip, [r10, 0x24]    ; stack limit
0xa3408c  ldr r2, [r10, 0x40]    ; null / sentinel from the thread
```

To get from a name to a function, the snapshot's own **class → method → code
range** table was read with an arm32 Dart-AOT parser. Its shipped whitelist
covered Dart 3.13 for 64-bit compressed-pointer builds only, so its arm32 grammar
was enabled locally and the same acceptance gate was run on this file before any
name was trusted: **Tier A 11/11, Tier B 3/3, 393,817 checks, grammar
consistent.** The recovered table gives one entry address per
method, and each address below was then opened a second time with the native
pseudo-C lifter (`native_function_pseudocode`). That lifter is register-level and
its ARM32 coverage stops at the VFP (floating-point) instructions, so where a
number is decided by `vmov`/`vldr`/`vstr` the same function was read from the
arm32 Tier-1 listing and the double was resolved in the snapshot's object pool.
Both readings are cited by address below.

The native lifter reports register-level pseudo-C (registers are not typed), so
the assembly was also checked instruction by instruction where a number matters.
Both readings are cited by address below.

### Numeric constants: what is readable and what is not

Plain IEEE-754 doubles are not stored as loose bytes; they live in the snapshot's
object pool. A raw byte search for `0.6`, `0.5` and `0.05` finds nothing. Once the
pool is decoded, the doubles used by the opacity code are readable:
`0.5` (the opacity default), `0.0` and `1.0` (the opacity clamp), `1.0` (the
starting scale) and `0.1` (the pencil coefficient). Values that are still
estimates are marked as such.

## Functions the rebuild now follows

| Function | Address | What it establishes |
| --- | --- | --- |
| `OpacityMixin` constructor | `0x1278698` | starts opacity at **0.5**, creates the **300 ms** (0x12c) settle timer, initialises the mixin's fields |
| `_OpacitySettingsState` pan/tap closures | `0x100a8c8`, `0x100a9b8`, `0x100aaa8` | opacity = `clamp(x / trackWidth, 0.0, 1.0)`; the two clamp doubles are 0.0 (pool `0x2797`) and 1.0 (pool `0xe67`) |
| `_OpacitySettingsState.build` | `0x100a158` | builds the opacity panel and reads the 0.0/1.0 constants |
| `_OpacitySettings@1874389511.build` | `0x128117c` | reads the AR opacity from field **0x1c** of the notifier |
| `_OpacitySettings@1380521617.build` | `0x126dba4` | the second opacity panel layout; reads a double from field 0x9c of the object it opens |
| `setInitialOpacity` | `0x12785b0` | writes the supplied double straight to `[this + 0x1c]`, no clamp |
| `changeOpacity` | `0x128129c` | writes the new double to `[this + 0x1c]`, runs the restorable callback and notifies listeners; **no clamp here** |
| `_OpacityButtonState.build` | `0xfbe3d4` | the quick opacity button picks **0.5** when its flag is clear and **1.0** when set |
| `CameraChangeNotifier` constructor | `0x12785c0` | starts the overlay scale field (0x44) at **1.0** |
| `setZoomState` | `0x110ed9c` | stores the active and previous zoom/scale preset (fields 0x3f / 0x43) and notifies listeners |
| `updateZoomState` (use case) | `0x128940c` | forwards the chosen scale to `setZoomState` and saves a snapshot |
| `_LockWidget.build` | `0x128c43c` | the lock button's closure calls `toggleLockedZoomState` |
| `toggleLockedZoomState` | `0x128c678` | locks the sheet transform by holding the current scale preset, or clears it |
| `toggleLight` (notifier) | `0x1007da0` | toggles the torch flag (field 0x67) and calls `setLightState` |
| `setLightState` | `0x1007e08` | applies the torch state to the camera |
| `FingerDrawingNotifier` constructor | `0x124400c` | default line type **brush**, size **1.0**, stroke opacity **1.0**, pencil coefficient **0.1** |
| `FingerDrawingNotifier.size` setter | `0x12b03f8` | writes the size field at **0x28** |
| `FingerDrawingNotifier.opacity` setter | `0x12b056c` | writes the stroke opacity field at **0x20** |
| `FingerDrawingNotifier.pencilCoefficient` | `0x12b024c` | writes the coefficient field at **0x40** |
| `_PencilAppearanceState` closure | `0xfa9b04` | pencil-size maths uses **20.0** as the upper size value |
| `ZoomSettings.build` | `0x128b154` | builds the scale-preset row |
| `ZoomSettingsItem.build` | `0x128b808` | renders one **0.5x / 1.0x / 2.0x** preset item |
| `OpacitySlider.build` | `0x12b0424` | the finger-drawing opacity slider; setter is `FingerDrawingNotifier.opacity` |
| `get:opacity` | `0x100a518` | reads the drawing notifier's opacity |

## The tracing screen (the app's core loop)

| Value | Value | Status |
| --- | --- | --- |
| Overlay opacity range | **0 % – 100 %** (0.0 – 1.0) | **confirmed** — clamp in `0x100a8c8` / `0x100a9b8`; pool doubles `0x2797` = 0.0, `0xe67` = 1.0 |
| Overlay opacity default | **50 %** (0.5) | **confirmed** — `OpacityMixin` constructor `0x1278698` |
| Opacity quick toggle | **50 % ↔ 100 %** | **confirmed** — `_OpacityButtonState.build` `0xfbe3d4` |
| Opacity settle | **300 ms** after the last change | **confirmed value, inferred role** — the timer object at `0x1278698` holds `0x12c`; the rebuild commits the default after that pause |
| Overlay scale range | **0.5× – 2.0×** | **confirmed** — labels `0.5x` (`0x8aacdc`), `1.0x` (`0x865d24`), `2.0x` (`0x721604`) and the `ZoomSettingsItem` row |
| Overlay scale default | **1.0×** | **confirmed** — `CameraChangeNotifier` constructor `0x12785c0` |
| Overlay rotation | free, −180°…180°, no snap | **estimated** |
| Pinch scale response | ratio of finger distance, clamped 0.9–1.1 per frame | **estimated** (standard pinch) |
| Two-finger rotation | half of the angle delta | **estimated** |
| Mouse-wheel scale | 0.15 % per wheel unit | **estimated** |
| Overlay sheet base size | 320 × 358 of the 430-wide screen | **estimated** from the instruction frames |
| Screen dim while a settings drawer is open | 22 % dark purple | **estimated** from the lilac tint in the instruction frames |
| Flashlight effect | warm 6 % white wash | **estimated** (torch in the original; a browser page cannot drive the torch) |
| Lock button | freezes the overlay transform | **confirmed feature / inferred effect** — `_LockWidget` `0x128c43c` calls `toggleLockedZoomState` `0x128c678`; help text `0x8b15ec` |
| Tool row | Opacity · Scaling · Center · Flashlight | **confirmed** by the strings above; `Center` replaces the earlier "Centric" |

### Opacity rule

```
opacity = clamp(pointer.x / track.width, 0.0, 1.0)
```

The sheet is drawn at that alpha. The quick button swaps between the two values
the original uses: **0.5** and **1.0**. The value is written to the notifier at
field **0x1c**; the slider itself passes an unclamped double, because the clamp
already happened in the two pan/tap closures. The rebuild uses the same 0–1
range, the same 50 % start, the same 50/100 toggle and a 300 ms settle before it
saves the new default.

### Scale rule

The original presents three scale labels — **0.5x, 1.0x and 2.0x** — and starts
at **1.0x**. The rebuild keeps a continuous slider for fine adjustment but adds
the three preset chips, and its **Center** action resets position and scale to
1.0× (the tool's own help line is "Center a sheet of paper in view").

## Step-by-step lessons

| Value | Value | Status |
| --- | --- | --- |
| Steps per sketch | **4** | **confirmed** — the app contains the counters `1/4` (`0x675e68`), `2/4` (`0x7bbea0`) and `3/4` (`0x74f640`), stored as UTF-16, and contains no `1/3`; `assets/step_by_step/step_1..step_4.webp` are four stages of one drawing |

The four shipped stage images were looked at: `step_1` is the bare outline,
`step_2`/`step_3` add the facial and hair detail, and `step_4` is the finished,
coloured drawing. The rebuild therefore counts 1…4 and gives stage 4 a soft
colour pass over the finished lines.

## Drawing tools

| Value | Value | Status |
| --- | --- | --- |
| Line types | **watercolor, dryWatercolor, dryWatercolor1, dryWatercolor2, brush, marker, pencil, eraser** | **confirmed** — the `MyLineEnum` instances carry these eight names |
| Default line type | **brush** | **confirmed** — `FingerDrawingNotifier` constructor `0x124400c` |
| Stroke opacity default | **1.0** | **confirmed** — field `0x20`, constructor `0x124400c` |
| Size default | **1.0** | **confirmed** — field `0x28`, constructor `0x124400c` |
| Pencil coefficient default | **0.1** | **confirmed** — field `0x40`; the double is `0x3FB999999999999A` (0.1) |
| Size range | 1 – 20 | **derived** — 20.0 appears in the pencil-size maths at `0xfa9b04`; the 1.0 lower end is the default |
| Pencil width | 1× size, alpha 1.00 | **estimated** |
| Marker width | 1.8× size, alpha 0.55 | **estimated** |
| Brush width | 1.35× size, alpha 0.90 | **estimated** |
| Eraser radius | 2.4× size | **estimated** |
| Colour palette | 10 swatches | **estimated**; the app ships its own palette art |
| Stroke sampling | a new point every 1.2 px | **estimated** |
| Undo depth | 30 snapshots | **estimated**; `Undo & Redo` (0x8733ec) confirms the feature |

The rebuild's tray names the four tools that fit its simplified panel
(Pencil / Marker / Brush / Eraser) and starts on **Brush** at size **1.0**, as
the original does. The four watercolour variants are documented but are not
separate buttons in this build.

## Rules of the loop

1. Choose a sketch (library, 10 built-in line drawings, Easy/Medium/Hard).
2. The sketch appears on a translucent sheet over the paper. Drag to move,
   pinch or slide **Scaling** to resize, or tap **0.5x / 1.0x / 2.0x**; use
   **Center** to recentre ("Center a sheet of paper in view").
3. Slide **Opacity** until the lines are easy to follow ("Here you can change
   the opacity of the image"). The quick button swaps **50 % ↔ 100 %**. Lock the
   sheet so it cannot move ("Lock the image perspective to prevent accidental
   moves").
4. Open the pencil button and trace with a finger. Brush is the original's
   default; Pencil / Marker / Brush / Eraser, 10 colours, size 1–20. Undo and
   Redo are always available.
5. **Flashlight** turns on the phone torch; if the room is dim the app points
   this out ("Switch on the flashlight if the lighting is low").
6. "Finish" opens the save card ("After completing your drawing, press Finish");
   saving adds the sketch, its strokes and the opacity to My drawings for this
   session.

## Other exact strings the rebuild now uses

| Text | Address | Where it is used |
| --- | --- | --- |
| `Opacity` | 0x5d283c (opacity_slider.dart) | opacity control |
| `Opacity Adjustment` | 0x5f3164 | opacity panel title |
| `Here you can change the opacity of the image` | 0x7015f4 | opacity panel help |
| `Adjust the slider to change opacity and close when done` | 0x6c837c | opacity panel hint |
| `Adjust zoom level and scale of the camera` | 0x6ac2d4 | zoom panel help |
| `Center a sheet of paper in view` | 0x6a7cec | Center tool help |
| `Center` | 0x690864 | tool label |
| `Scaling` | 0x84005c | scale control label |
| `Flashlight` | 0x89e4fc | tool label |
| `Turn on the flashlight for better lighting` | 0x730414 | torch-on tip |
| `Switch on the flashlight if the lighting is low` | 0x618214 | low-light tip |
| `Brighten the room or use Flashlight` | 0x83633c | AI-feedback tip |
| `Tap to change transparency` / `Swipe to change transparency` | 0x76548c / 0x79b22c | transparency gestures |
| `Undo & Redo` | 0x8733ec | drawing actions |
| `Lock the drawing on the screen` | 0x7d5ab4 | lock hint |
| `Lock the image perspective to prevent accidental moves` | 0x8b15ec | lock help |
| `After completing your drawing, press Finish` | 0x4f9584 | finish hint |
| `0.5x`, `1.0x`, `2.0x` | 0x8aacdc, 0x865d24, 0x721604 | scale presets |

Control assets confirm the same set: `opacity_icon.svg`, `zoom_icon.svg`,
`zoom_icon_active.svg`, `flashlight_of.svg`, `arrow_undo.svg`,
`arrow_redo.svg`, `steps_icon.svg`, all under `assets/vector/`.

## What the original has that this build does not

| Original | This build | Why |
| --- | --- | --- |
| Live camera passthrough behind the sheet | Drawn desk-and-paper view (the web page can show the phone camera behind a transparent canvas when the browser allows it) | macroquad has no camera API; the web page supplies a `<video>` layer |
| Photo upload from the gallery | 10 built-in sketches | no file picker in macroquad |
| AI photo-to-sketch, 800+ sketches, 148 step-by-step lessons, coloring book, paywall | one sketch library + a four-step mode | online/paywalled features are out of scope for a single offline page |
| Accounts, album cloud, reminders | session-only gallery | no backend |
| Four watercolour line variants | Pencil / Marker / Brush / Eraser | the simplified tray has four slots; the enum is documented above |

## What static analysis still cannot answer

- The exact per-tool texture rendering (the app ships `assets/finger_drawing/textures/part_*.webp`); the rebuild approximates the line with width and alpha.
- The exact effect of `pencilCoefficient` on the rendered stroke; the default 0.1 is real, the visual mapping is not recovered.
- Pinch sensitivity, wheel step, the overlay's 320 × 358 base size and the screen dim are still estimates.
- Which camera frames the app shows before the first permission answer, and how the torch behaves on a browser page (no torch API).
