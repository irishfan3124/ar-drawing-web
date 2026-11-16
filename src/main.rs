// AR Drawing â€” a browser-and-desktop rebuild of the AR tracing studio.
// Layout and interaction notes live in MECHANICS.md next to this project.

mod art;
mod ui;

use art::{draw_art, samples, Art};
use macroquad::prelude::*;
use ui::*;

const ACCENT1: Color = Color::new(1.0, 0.416, 0.557, 1.0);
const ACCENT2: Color = Color::new(0.992, 0.310, 0.467, 1.0);

// Read out of libapp.so: the AR opacity field is clamped 0.0..1.0 by
// _OpacitySettingsState's tap/pan closures (0x100a8c8 and 0x100a9b8,
// pool doubles at 0x2797 = 0.0 and 0xe67 = 1.0).
const OPACITY_MIN: f32 = 0.0;
const OPACITY_MAX: f32 = 1.0;
// OpacityMixin's constructor (0x1278698) starts at 0.5 and creates a 300 ms
// (0x12c) settle timer. The opacity quick button toggles 0.5 <-> 1.0.
const OPACITY_DEFAULT: f32 = 0.5;
const OPACITY_QUICK_LOW: f32 = 0.5;
const OPACITY_QUICK_HIGH: f32 = 1.0;
const OPACITY_SETTLE_SECS: f32 = 0.3;
// CameraChangeNotifier's constructor (0x12785c0) starts the sheet at 1.0x.
// The zoom labels shipped by the app are "0.5x", "1.0x" and "2.0x".
const SCALE_MIN: f32 = 0.5;
const SCALE_MAX: f32 = 2.0;
const SCALE_PRESETS: [f32; 3] = [0.5, 1.0, 2.0];
// FingerDrawingNotifier's constructor (0x124400c) starts size at 1.0 and
// pencilCoefficient at 0.1; the pencil-size maths in _PencilAppearanceState
// (0xfa9b04) uses 20.0 as its upper value.
const SIZE_MIN: f32 = 1.0;
const SIZE_MAX: f32 = 20.0;
const SIZE_DEFAULT: f32 = 1.0;
const PENCIL_COEFFICIENT_DEFAULT: f32 = 0.1;

const PHOTO_TOP: f32 = 96.0;
const PHOTO_H: f32 = 620.0;
const SHEET_Y: f32 = 600.0;

const STROKE_COLORS: [Color; 10] = [
    Color::new(0.992, 0.310, 0.467, 1.0),
    Color::new(0.902, 0.224, 0.275, 1.0),
    Color::new(0.976, 0.549, 0.239, 1.0),
    Color::new(0.984, 0.804, 0.188, 1.0),
    Color::new(0.353, 0.718, 0.365, 1.0),
    Color::new(0.176, 0.678, 0.643, 1.0),
    Color::new(0.235, 0.478, 0.882, 1.0),
    Color::new(0.545, 0.373, 0.812, 1.0),
    Color::new(0.471, 0.353, 0.263, 1.0),
    Color::new(0.117, 0.110, 0.129, 1.0),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Library,
    Gallery,
    Settings,
    Trace,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    None,
    Opacity,
    Scaling,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Modal {
    None,
    Finish,
    About,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Pencil,
    Marker,
    Brush,
    Eraser,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Tool::Pencil => "Pencil",
            Tool::Marker => "Marker",
            Tool::Brush => "Brush",
            Tool::Eraser => "Eraser",
        }
    }
}

#[derive(Clone)]
struct Stroke {
    pts: Vec<Vec2>,
    color: Color,
    width: f32,
    alpha: f32,
    tool: Tool,
}

struct Saved {
    art: usize,
    strokes: Vec<Stroke>,
    opacity: f32,
    seconds: f32,
    ago: f32,
}

struct Trace {
    art: usize,
    pos: Vec2,
    scale: f32,
    rot: f32,
    opacity: f32,
    opacity_settle: f32,
    locked: bool,
    flashlight: bool,
    panel: Panel,
    tool: Tool,
    color: usize,
    size: f32,
    pencil_coefficient: f32,
    strokes: Vec<Stroke>,
    redo: Vec<Stroke>,
    undo_stack: Vec<Vec<Stroke>>,
    current: Option<Stroke>,
    sheet: bool,
    hide_ui: bool,
    step_mode: bool,
    step: u8,
    started: f32,
    dragging: bool,
    drag_off: Vec2,
    pinch: Option<(f32, f32, Vec2)>,
}

impl Trace {
    fn new(art: usize, step_mode: bool, opacity: f32) -> Self {
        Self {
            art,
            pos: vec2(VW / 2.0, 415.0),
            scale: 1.0,
            rot: 0.0,
            opacity,
            opacity_settle: 0.0,
            locked: false,
            flashlight: false,
            panel: Panel::None,
            tool: Tool::Brush,
            color: 9,
            size: SIZE_DEFAULT,
            pencil_coefficient: PENCIL_COEFFICIENT_DEFAULT,
            strokes: Vec::new(),
            redo: Vec::new(),
            undo_stack: Vec::new(),
            current: None,
            sheet: false,
            hide_ui: false,
            step_mode,
            step: if step_mode { 1 } else { 4 },
            started: 0.0,
            dragging: false,
            drag_off: Vec2::ZERO,
            pinch: None,
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.strokes.clone());
        if self.undo_stack.len() > 30 {
            self.undo_stack.remove(0);
        }
        self.redo.clear();
    }
}

#[derive(Clone, Copy, Default)]
struct Input {
    pos: Vec2,
    last: Vec2,
    delta: Vec2,
    down: bool,
    pressed: bool,
    released: bool,
    wheel: f32,
    second: Option<Vec2>,
    /// Set once the platform has delivered a real touch. Phones never move the
    /// mouse, so from then on the touch is the only trustworthy pointer.
    touch: bool,
}

#[derive(Clone, Copy, Default)]
struct Hud {
    title: bool,
    finish: bool,
    opacity: bool,
    tools: bool,
}

#[derive(Clone)]
struct SelfTest {
    frame: u32,
    results: Vec<(&'static str, bool, String)>,
    start_pos: Vec2,
    moved: f32,
    done: bool,
}

enum Act {
    Home,
    Library,
    Gallery,
    Settings,
    Open(usize),
    StepOpen(usize),
    Back,
    Panel(Panel),
    OpacityToggle,
    ScalePreset(f32),
    Tool(Tool),
    Color(usize),
    Undo,
    Redo,
    Clear,
    ToggleSheet,
    ToggleHide,
    CloseSheet,
    ToggleLock,
    ToggleFlash,
    Center,
    Guide,
    NextStep,
    PrevStep,
    AskFinish,
    KeepDrawing,
    Save,
    Filter(usize),
    ResetGallery,
    CloseModal,
}

struct App {
    fonts: Fonts,
    arts: Vec<Art>,
    screen: Screen,
    prev_screen: Screen,
    filter: usize,
    lib_scroll: f32,
    trace: Trace,
    gallery: Vec<Saved>,
    default_opacity: f32,
    guide: bool,
    toast: Option<(String, f32)>,
    input: Input,
    act: Option<Act>,
    time: f32,
    hud: Hud,
    modal: Modal,
    st: Option<SelfTest>,
    slider_drag: Option<u32>,
    press_pos: Vec2,
    anim: f32,
}

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "AR Drawing".to_owned(),
            window_width: 430,
            window_height: 900,
            high_dpi: cfg!(target_arch = "wasm32"),
            window_resizable: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

static PREVIEW: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn parse_args() -> (Option<String>, Option<String>) {
    let mut shot = None;
    let mut test = None;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            shot = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--self-test" && i + 1 < args.len() {
            test = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--preview" && i + 1 < args.len() {
            let _ = PREVIEW.set(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }
    (shot, test)
}

#[cfg(target_os = "windows")]
fn sink_window() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetWindowPos, ShowWindow, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE,
    };
    let title: Vec<u16> = OsStr::new("AR Drawing")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            // Hide, then show without activating, then park it at the bottom of the stack.
            ShowWindow(hwnd, SW_HIDE);
            SetWindowPos(
                hwnd,
                HWND_BOTTOM,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn sink_window() {}

fn save_screenshot(path: &str) {
    // The window draws upright now, so the readback is stored exactly as the
    // renderer gives it; Image::export_png does the one row flip a PNG needs.
    let src = get_screen_data();
    let mut out = Image::gen_image_color(src.width, src.height, BLANK);
    out.bytes.copy_from_slice(&src.bytes);
    out.export_png(path);
}

#[macroquad::main(window_conf)]
async fn main() {
    let (shot_path, test_path) = parse_args();
    let mut app = App::new();
    if test_path.is_some() {
        app.st = Some(SelfTest {
            frame: 0,
            results: Vec::new(),
            start_pos: app.trace.pos,
            moved: 0.0,
            done: false,
        });
    }
    let mut frame: u32 = 0;
    loop {
        let dt = get_frame_time().min(0.05);
        app.time += dt;
        if let Some(st) = app.st.as_ref() {
            if !st.done {
                app.script();
            }
        } else {
            app.poll_input();
        }
        app.update(dt);
        app.draw();

        frame += 1;
        if frame == 2 && shot_path.is_some() {
            sink_window();
        }
        if let Some(path) = &shot_path {
            if frame == 120 {
                save_screenshot(path);
                std::process::exit(0);
            }
        }
        if let Some(path) = &test_path {
            if app.st.as_ref().map(|s| s.done).unwrap_or(false) {
                app.write_self_test(path);
                std::process::exit(0);
            }
        }
        next_frame().await
    }
}

impl App {
    fn new() -> Self {
        let fonts = Fonts::load();
        let arts = samples();
        let mut app = Self {
            fonts,
            arts,
            screen: Screen::Home,
            prev_screen: Screen::Home,
            filter: 0,
            lib_scroll: 0.0,
            trace: Trace::new(0, false, OPACITY_DEFAULT),
            gallery: Vec::new(),
            default_opacity: OPACITY_DEFAULT,
            guide: false,
            toast: None,
            input: Input::default(),
            act: None,
            time: 0.0,
            hud: Hud::default(),
            modal: Modal::None,
            st: None,
            slider_drag: None,
            press_pos: Vec2::ZERO,
            anim: 0.0,
        };
        if let Some(p) = PREVIEW.get() {
            match p.as_str() {                "library" => app.screen = Screen::Library,
                "gallery" => {
                    app.screen = Screen::Gallery;
                    let mut s1: Vec<Vec2> = Vec::new();
                    let mut s2: Vec<Vec2> = Vec::new();
                    for i in 0..40 {
                        let t = i as f32 / 39.0;
                        s1.push(vec2(0.28 + 0.34 * t, 0.34 + 0.04 * (t * 6.0).sin()));
                        s2.push(vec2(0.36 + 0.22 * t, 0.66 + 0.03 * (t * 5.0).cos()));
                    }
                    app.gallery.push(Saved {
                        art: 1,
                        strokes: vec![
                            Stroke { pts: s1, color: STROKE_COLORS[9], width: 3.0, alpha: 1.0, tool: Tool::Pencil },
                            Stroke { pts: s2, color: STROKE_COLORS[6], width: 8.0, alpha: 0.55, tool: Tool::Marker },
                        ],
                        opacity: OPACITY_DEFAULT,
                        seconds: 90.0,
                        ago: 20.0,
                    });
                    app.gallery.push(Saved {
                        art: 2,
                        strokes: Vec::new(),
                        opacity: 0.5,
                        seconds: 120.0,
                        ago: 200.0,
                    });
                }
                "settings" => app.screen = Screen::Settings,
                "trace" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(0, false, OPACITY_DEFAULT);
                    app.trace.started = 0.0;
                }
                "opacity" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(0, false, OPACITY_DEFAULT);
                    app.trace.panel = Panel::Opacity;
                    app.trace.started = 0.0;
                }
                "scaling" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(0, false, OPACITY_DEFAULT);
                    app.trace.panel = Panel::Scaling;
                    app.trace.started = 0.0;
                }
                "sheet" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(2, false, OPACITY_DEFAULT);
                    app.trace.sheet = true;
                    app.trace.started = 0.0;
                }
                "steps" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(3, true, OPACITY_DEFAULT);
                    app.trace.step = 4;
                    app.trace.started = 0.0;
                }
                "finish" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(1, false, OPACITY_DEFAULT);
                    app.modal = Modal::Finish;
                    app.trace.started = 0.0;
                }
                "drawn" => {
                    app.screen = Screen::Trace;
                    app.trace = Trace::new(0, false, OPACITY_DEFAULT);
                    app.trace.started = 0.0;
                    let mut a: Vec<Vec2> = Vec::new();
                    let mut b: Vec<Vec2> = Vec::new();
                    for i in 0..40 {
                        let t = i as f32 / 39.0;
                        a.push(vec2(150.0 + 120.0 * t, 330.0 + 26.0 * (t * 6.0).sin()));
                        b.push(vec2(165.0 + 90.0 * t, 500.0 + 18.0 * (t * 5.0).cos()));
                    }
                    app.trace.strokes.push(Stroke {
                        pts: a,
                        color: STROKE_COLORS[9],
                        width: 3.0,
                        alpha: 1.0,
                        tool: Tool::Pencil,
                    });
                    app.trace.strokes.push(Stroke {
                        pts: b,
                        color: STROKE_COLORS[6],
                        width: 8.0,
                        alpha: 0.55,
                        tool: Tool::Marker,
                    });
                }
                _ => {}
            }
        }
        #[cfg(webshot)]
        {
            app.screen = Screen::Trace;
            app.trace = Trace::new(0, false, OPACITY_DEFAULT);
            app.trace.started = 0.0;
        }
        app
    }

    fn view(&self) -> Rect {
        let s = (screen_width() / VW).min(screen_height() / VH);
        let vw = screen_width() / s;
        let vh = screen_height() / s;
        Rect::new(VW / 2.0 - vw / 2.0, VH / 2.0 - vh / 2.0, vw, vh)
    }

    fn to_virtual(&self, p: Vec2) -> Vec2 {
        let v = self.view();
        vec2(
            v.x + p.x * v.w / screen_width(),
            v.y + p.y * v.h / screen_height(),
        )
    }

    fn poll_input(&mut self) {
        let view = self.view();
        let scale = view.w / screen_width();
        let mp = mouse_position();
        let mut pos = vec2(view.x + mp.0 * scale, view.y + mp.1 * scale);
        let mut down = is_mouse_button_down(MouseButton::Left);
        let mut second = None;
        let ts = touches();
        if !ts.is_empty() {
            let map = |p: Vec2| vec2(view.x + p.x * scale, view.y + p.y * scale);
            pos = map(ts[0].position);
            down = true;
            self.input.touch = true;
            if ts.len() > 1 {
                second = Some(map(ts[1].position));
            }
        } else if self.input.touch {
            // The frame a finger lifts, `touches()` is already empty and the mouse
            // fallback is worthless on a phone: no mouse event ever moved it, so
            // the release point would snap to the top-left corner and no button
            // would match its rectangle. Hold the last real touch point instead.
            pos = self.input.pos;
            down = false;
        }
        let pressed = down && !self.input.down;
        let released = !down && self.input.down;
        self.input.last = self.input.pos;
        self.input.pos = pos;
        if pressed {
            self.press_pos = pos;
        }
        self.input.delta = pos - self.input.last;
        self.input.down = down;
        self.input.pressed = pressed;
        self.input.released = released;
        self.input.wheel = mouse_wheel().1;
        self.input.second = second;
    }

    fn hover(&self, r: Rect) -> bool {
        r.contains(self.input.pos)
    }

    fn clicked(&self, r: Rect) -> bool {
        self.input.pressed && r.contains(self.input.pos)
    }

    fn button(&self, r: Rect) -> bool {
        if self.modal != Modal::None && !(r.y >= 216.0 && r.y + r.h <= 626.0) {
            return false;
        }
        self.input.released
            && r.contains(self.input.pos)
            && r.contains(self.press_pos)
            && (self.input.pos - self.press_pos).length() < 14.0
    }

    fn slider(&mut self, id: u32, track: Rect, v: &mut f32, min: f32, max: f32) -> bool {
        let hit = self.input.pos.x > track.x - 18.0
            && self.input.pos.x < track.x + track.w + 18.0
            && self.input.pos.y > track.y - 22.0
            && self.input.pos.y < track.y + 22.0;
        if self.input.pressed && hit {
            self.slider_drag = Some(id);
        }
        if !self.input.down {
            if self.slider_drag == Some(id) {
                self.slider_drag = None;
            }
        }
        if self.slider_drag == Some(id) {
            let t = ((self.input.pos.x - track.x) / track.w).clamp(0.0, 1.0);
            let nv = min + (max - min) * t;
            let changed = (nv - *v).abs() > 0.0001;
            *v = nv;
            return changed;
        }
        false
    }

    fn set_act(&mut self, a: Act) {
        self.act = Some(a);
    }

    fn update(&mut self, dt: f32) {
        self.anim += dt;
        if let Some((_, t)) = &mut self.toast {
            *t -= dt;
            if *t <= 0.0 {
                self.toast = None;
            }
        }
        // The original keeps a 300 ms settle timer next to OpacityMixin's
        // opacity field (0x1278698); commit the last value only after it rests.
        if self.trace.opacity_settle > 0.0 {
            self.trace.opacity_settle -= dt;
            if self.trace.opacity_settle <= 0.0 {
                self.default_opacity = self.trace.opacity;
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            if self.modal != Modal::None {
                self.modal = Modal::None;
            } else if self.screen == Screen::Trace {
                self.set_act(Act::Back);
            } else if self.screen != Screen::Home {
                self.set_act(Act::Home);
            }
        }
        if let Some(a) = self.act.take() {
            self.apply(a);
        }
    }

    fn apply(&mut self, a: Act) {
        match a {
            Act::Home => {
                self.screen = Screen::Home;
                self.modal = Modal::None;
            }
            Act::Library => {
                self.prev_screen = self.screen;
                self.screen = Screen::Library;
                self.modal = Modal::None;
            }
            Act::Gallery => {
                self.screen = Screen::Gallery;
                self.modal = Modal::None;
            }
            Act::Settings => {
                self.screen = Screen::Settings;
                self.modal = Modal::None;
            }
            Act::Open(i) => {
                self.trace = Trace::new(i, false, self.default_opacity);
                self.trace.started = self.time;
                self.screen = Screen::Trace;
                self.modal = Modal::None;
            }
            Act::StepOpen(i) => {
                self.trace = Trace::new(i, true, self.default_opacity);
                self.trace.started = self.time;
                self.screen = Screen::Trace;
                self.modal = Modal::None;
            }
            Act::Back => {
                if self.trace.sheet {
                    self.trace.sheet = false;
                } else if self.modal != Modal::None {
                    self.modal = Modal::None;
                } else {
                    self.screen = Screen::Library;
                }
            }
            Act::Panel(p) => {
                self.trace.panel = if self.trace.panel == p { Panel::None } else { p };
                self.trace.sheet = false;
            }
            Act::OpacityToggle => {
                // _OpacityButtonState.build (0xfbe3d4) picks 0.5 when the
                // quick-opacity flag is unset and 1.0 when it is set.
                self.trace.opacity = if (self.trace.opacity - OPACITY_QUICK_LOW).abs() < 0.01 {
                    OPACITY_QUICK_HIGH
                } else {
                    OPACITY_QUICK_LOW
                };
                self.trace.opacity_settle = OPACITY_SETTLE_SECS;
                self.toast = Some((
                    format!("Opacity {:.0}%", self.trace.opacity * 100.0),
                    1.2,
                ));
            }
            Act::ScalePreset(v) => {
                self.trace.scale = v;
                self.toast = Some((format!("Scale {:.1}x", v), 1.2));
            }
            Act::Tool(t) => {
                self.trace.tool = t;
                self.trace.sheet = true;
            }
            Act::Color(i) => self.trace.color = i,
            Act::Undo => {
                if let Some(prev) = self.trace.undo_stack.pop() {
                    self.trace.redo = self.trace.strokes.clone();
                    self.trace.strokes = prev;
                }
            }
            Act::Redo => {
                if !self.trace.redo.is_empty() {
                    let r = std::mem::take(&mut self.trace.redo);
                    self.trace.undo_stack.push(self.trace.strokes.clone());
                    self.trace.strokes = r;
                }
            }
            Act::Clear => {
                if !self.trace.strokes.is_empty() {
                    self.trace.push_undo();
                    self.trace.strokes.clear();
                }
            }
            Act::ToggleSheet => {
                self.trace.sheet = !self.trace.sheet;
                self.trace.panel = Panel::None;
            }
            Act::ToggleHide => {
                self.trace.hide_ui = !self.trace.hide_ui;
                if self.trace.hide_ui {
                    self.trace.sheet = false;
                    self.trace.panel = Panel::None;
                }
            }
            Act::CloseSheet => self.trace.sheet = false,
            Act::ToggleLock => {
                self.trace.locked = !self.trace.locked;
                // Exact hint strings from libapp.so: "Lock the drawing on the
                // screen" (0x7d5ab4) and "Lock the image perspective to prevent
                // accidental moves" (0x8b15ec). _LockWidget calls
                // toggleLockedZoomState (0x128c678).
                self.toast = Some((
                    if self.trace.locked {
                        "Lock the image perspective to prevent accidental moves".into()
                    } else {
                        "Lock the drawing on the screen".into()
                    },
                    1.8,
                ));
            }
            Act::ToggleFlash => {
                self.trace.flashlight = !self.trace.flashlight;
                if self.trace.flashlight {
                    // The original's own tip, at 0x730414 in libapp.so.
                    self.toast = Some(("Turn on the flashlight for better lighting".into(), 1.8));
                }
            }
            Act::Center => {
                self.trace.pos = vec2(VW / 2.0, 415.0);
                self.trace.scale = 1.0;
                self.trace.rot = 0.0;
                self.toast = Some(("Picture centred".into(), 1.6));
            }
            Act::Guide => {
                self.guide = !self.guide;
                self.toast = Some((
                    if self.guide { "Guide grid on".into() } else { "Guide grid off".into() },
                    1.4,
                ));
            }
            Act::NextStep => {
                self.trace.step = (self.trace.step + 1).min(4);
            }
            Act::PrevStep => {
                self.trace.step = self.trace.step.saturating_sub(1).max(1);
            }
            Act::AskFinish => self.modal = Modal::Finish,
            Act::KeepDrawing => self.modal = Modal::None,
            Act::Save => {
                let mut strokes = Vec::new();
                for s in &self.trace.strokes {
                    let pts: Vec<Vec2> = s
                        .pts
                        .iter()
                        .map(|p| {
                            vec2(
                                (p.x / VW).clamp(0.0, 1.0),
                                ((p.y - PHOTO_TOP) / PHOTO_H).clamp(0.0, 1.0),
                            )
                        })
                        .collect();
                    strokes.push(Stroke { pts, ..s.clone() });
                }
                self.gallery.insert(
                    0,
                    Saved {
                        art: self.trace.art,
                        strokes,
                        opacity: self.trace.opacity,
                        seconds: (self.time - self.trace.started).max(1.0),
                        ago: 0.0,
                    },
                );
                self.modal = Modal::None;
                self.toast = Some(("Saved to My drawings".into(), 2.0));
                self.screen = Screen::Gallery;
            }
            Act::Filter(i) => {
                self.filter = i;
                self.lib_scroll = 0.0;
            }
            Act::ResetGallery => {
                self.gallery.clear();
                self.toast = Some(("Gallery cleared".into(), 1.6));
            }
            Act::CloseModal => self.modal = Modal::None,
        }
    }

    // ------------------------------------------------------------ self test

    fn script(&mut self) {
        let f = self.st.as_ref().unwrap().frame;
        self.input.second = None;
        let mut pos = self.input.pos;
        let mut down = false;
        match f {
            6 => {
                pos = vec2(109.0, 292.0); // Start drawing
                down = true;
            }
            16 => {
                pos = vec2(113.0, 268.0); // first sketch card
                down = true;
            }
            20..=54 => {
                let t = ((f - 20) as f32 / 34.0).clamp(0.0, 1.0);
                pos = vec2(215.0, 415.0).lerp(vec2(283.0, 352.0), t);
                down = true;
            }
            58 => {
                pos = vec2(388.0, 668.0); // pencil button
                down = true;
            }
            64..=108 => {
                let t = ((f - 64) as f32 / 44.0).clamp(0.0, 1.0);
                let x = 150.0 + 120.0 * t;
                let y = 400.0 + 110.0 * t + (t * std::f32::consts::TAU).sin() * 8.0;
                pos = vec2(x, y);
                down = true;
            }
            112 => {
                pos = vec2(407.0, 633.0); // close the drawing sheet
                down = true;
            }
            _ => {}
        }
        let pressed = down && !self.input.down;
        let released = !down && self.input.down;
        self.input.last = self.input.pos;
        self.input.pos = pos;
        if pressed {
            self.press_pos = pos;
        }
        self.input.delta = if pressed { Vec2::ZERO } else { pos - self.input.last };
        self.input.down = down;
        self.input.pressed = pressed;
        self.input.released = released;
        self.input.wheel = 0.0;

        if f == 20 {
            self.st.as_mut().unwrap().start_pos = self.trace.pos;
        }
        if f == 58 {
            let moved = (self.trace.pos - self.st.as_ref().unwrap().start_pos).length();
            self.st.as_mut().unwrap().moved = moved;
            let ok = moved > 20.0 && self.trace.art == 0;
            self.st.as_mut().unwrap().results.push((
                "move",
                ok,
                format!(
                    "dragged the picture {:.0} px across the page (expected at least 20)",
                    moved
                ),
            ));
        }
        if f == 110 {
            let strokes = self.trace.strokes.len();
            let pts: usize = self.trace.strokes.iter().map(|s| s.pts.len()).sum();
            let ok = strokes >= 1 && pts >= 8;
            self.st.as_mut().unwrap().results.push((
                "act",
                ok,
                format!(
                    "drew {} stroke(s) with {} points at {:.0}% picture opacity",
                    strokes,
                    pts,
                    self.trace.opacity * 100.0
                ),
            ));
        }
        if f == 118 {
            let h = self.hud;
            let ok = h.title && h.finish && h.opacity && h.tools;
            self.st.as_mut().unwrap().results.push((
                "hud",
                ok,
                format!(
                    "top bar title {}, Finish {}, opacity slider {}, four tool buttons {}",
                    yes(h.title),
                    yes(h.finish),
                    yes(h.opacity),
                    yes(h.tools)
                ),
            ));
            self.st.as_mut().unwrap().done = true;
        }
        self.st.as_mut().unwrap().frame += 1;
    }

    fn write_self_test(&self, path: &str) {
        let st = self.st.as_ref().unwrap();
        let mut s = String::from("{\"version\": 1, \"steps\": [");
        for (i, (name, ok, detail)) in st.results.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&format!(
                "{{\"step\": \"{}\", \"passed\": {}, \"detail\": \"{}\"}}",
                name,
                ok,
                detail.replace('"', "'")
            ));
        }
        s.push_str("]}");
        let _ = std::fs::write(path, s);
    }

    // ---------------------------------------------------------------- draw

    fn draw(&mut self) {
        let v = self.view();
        #[cfg(target_arch = "wasm32")]
        let bg = if self.screen == Screen::Trace {
            Color::new(0.0, 0.0, 0.0, 0.0)
        } else {
            rgba(250.0, 244.0, 247.0, 1.0)
        };
        #[cfg(not(target_arch = "wasm32"))]
        let bg = rgba(250.0, 244.0, 247.0, 1.0);
        clear_background(bg);
        // This renderer hands back a bottom-up framebuffer, so screen space has to
        // be flipped to land on the window the right way up. The same camera is
        // used on desktop and in the browser: one orientation, one set of numbers.
        let mut cam = Camera2D::from_display_rect(v);
        cam.zoom.y = -cam.zoom.y;
        set_camera(&cam);
        self.act = None;
        match self.screen {
            Screen::Home => self.draw_home(),
            Screen::Library => self.draw_library(),
            Screen::Gallery => self.draw_gallery(),
            Screen::Settings => self.draw_settings(),
            Screen::Trace => self.draw_trace(),
        }
        self.draw_toast();
        set_default_camera();
    }

    fn draw_toast(&mut self) {
        if let Some((msg, t)) = &self.toast {
            let a = (t * 2.5).min(1.0);
            let w = text_w(&self.fonts.semi, msg, 12.0) + 44.0;
            let x = (VW - w) / 2.0;
            let y = 836.0;
            shadow(x, y, w, 36.0, 18.0, 0.5 * a);
            rrect(x, y, w, 36.0, 18.0, fade(ACCENT, a));
            text_center(&self.fonts.semi, msg, VW / 2.0, y + 23.0, 12.0, fade(SNOW, a));
        }
    }

    fn nav_bar(&mut self) {
        let y = 816.0;
        shadow(0.0, y, VW, 84.0, 0.0, 0.35);
        rect(0.0, y, VW, 84.0, SNOW);
        draw_line(0.0, y, VW, y, 1.0, rgba(0.0, 0.0, 0.0, 0.05));
        let items = [
            ("Draw", Screen::Home, 0),
            ("Gallery", Screen::Gallery, 1),
            ("Settings", Screen::Settings, 2),
        ];
        for (i, (label, target, kind)) in items.iter().enumerate() {
            let cx = 72.0 + i as f32 * 143.0;
            let r = Rect::new(cx - 55.0, y, 110.0, 84.0);
            let active = self.screen == *target;
            let col = if active { ACCENT } else { GREY_LIGHT };
            let cy = y + 32.0;
            match kind {
                0 => icon_home(vec2(cx, cy), 22.0, 2.0, col),
                1 => icon_gallery(vec2(cx, cy), 22.0, 2.0, col),
                _ => icon_sliders(vec2(cx, cy), 20.0, 1.8, col),
            }
            text_center(
                &self.fonts.semi,
                label,
                cx,
                y + 62.0,
                10.0,
                if active { ACCENT } else { GREY },
            );
            if self.button(r) {
                self.set_act(match target {
                    Screen::Home => Act::Home,
                    Screen::Gallery => Act::Gallery,
                    _ => Act::Settings,
                });
            }
        }
    }

    fn draw_home(&mut self) {
        let t = self.anim;
        // pink header
        vgrad(0.0, 0.0, VW, 236.0, ACCENT1, ACCENT2);
        rrect(0.0, 196.0, VW, 40.0, 24.0, ACCENT2);
        draw_circle(348.0, 52.0, 86.0, rgba(255.0, 255.0, 255.0, 0.06));
        draw_circle(44.0, 196.0, 70.0, rgba(255.0, 255.0, 255.0, 0.05));

        // app mark
        rrect(22.0, 56.0, 40.0, 40.0, 12.0, SNOW);
        rrect(26.0, 60.0, 32.0, 32.0, 9.0, ACCENT);
        rrect(32.0, 66.0, 20.0, 20.0, 5.0, SNOW);
        icon_pencil(vec2(43.0, 77.0), 17.0, 1.6, ACCENT);

        text(&self.fonts.bold, "AR Drawing", 72.0, 84.0, 21.0, SNOW);
        text(&self.fonts.regular, "Trace anything, anywhere", 72.0, 100.0, 10.5, rgba(255.0, 255.0, 255.0, 0.82));

        let gear = Rect::new(378.0, 62.0, 34.0, 34.0);
        rrect(gear.x, gear.y, gear.w, gear.h, 12.0, rgba(255.0, 255.0, 255.0, 0.16));
        icon_sliders(vec2(395.0, 79.0), 17.0, 1.6, SNOW);
        if self.button(gear) {
            self.set_act(Act::Settings);
        }

        // hero
        let hero = Rect::new(18.0, 150.0, 394.0, 178.0);
        shadow(hero.x, hero.y, hero.w, hero.h, 24.0, 1.0);
        rrect(hero.x, hero.y, hero.w, hero.h, 24.0, SNOW);
        text(&self.fonts.bold, "Draw anything with", 38.0, 198.0, 19.0, INK);
        text(&self.fonts.bold, "augmented reality!", 38.0, 219.0, 19.0, INK);
        text(
            &self.fonts.regular,
            "Pick a sketch, place your phone,",
            38.0,
            242.0,
            10.0,
            GREY,
        );
        text(&self.fonts.regular, "then trace it on paper.", 38.0, 256.0, 10.0, GREY);

        let btn = Rect::new(38.0, 272.0, 148.0, 42.0);
        rrect(btn.x, btn.y + 2.0, btn.w, btn.h, 21.0, rgba(253.0, 79.0, 119.0, 0.20));
        rrect(btn.x, btn.y, btn.w, btn.h, 21.0, ACCENT);
        icon_play(vec2(btn.x + 24.0, btn.y + 21.0), 14.0, SNOW);
        text(&self.fonts.semi, "Start drawing", btn.x + 40.0, btn.y + 26.0, 12.0, SNOW);
        if self.button(btn) {
            self.set_act(Act::Library);
        }

        // mini preview
        let bob = (t * 1.4).sin() * 1.6;
        rrect(296.0, 184.0 + bob, 96.0, 110.0, 12.0, rgba(255.0, 246.0, 249.0, 1.0));
        rrect_line(296.0, 184.0 + bob, 96.0, 110.0, 12.0, 1.0, rgba(253.0, 79.0, 119.0, 0.25));
        draw_art(
            &self.arts[0],
            vec2(344.0, 236.0 + bob),
            86.0,
            -0.12,
            0.95,
            rgba(90.0, 82.0, 96.0, 1.0),
            0.75,
            3,
        );

        // section
        text(&self.fonts.bold, "Choose a drawing", 20.0, 370.0, 15.0, INK);
        text_right(&self.fonts.semi, "See all", 410.0, 370.0, 11.0, ACCENT);
        if self.button(Rect::new(350.0, 350.0, 70.0, 28.0)) {
            self.set_act(Act::Library);
        }

        let cards = [
            ("Sketch library", 0),
            ("Step by step", 1),
            ("Coloring", 2),
        ];
        for (i, (label, kind)) in cards.iter().enumerate() {
            let x = 18.0 + i as f32 * 135.0;
            let r = Rect::new(x, 388.0, 124.0, 108.0);
            shadow(r.x, r.y, r.w, r.h, 18.0, 0.8);
            rrect(r.x, r.y, r.w, r.h, 18.0, SNOW);
            rrect(r.x + 44.0, r.y + 16.0, 36.0, 36.0, 12.0, ACCENT_SOFT);
            let c = vec2(r.x + 62.0, r.y + 34.0);
            match kind {
                0 => icon_photo(c, 20.0, 1.8, ACCENT),
                1 => icon_grid(c, 19.0, 1.8, ACCENT),
                _ => icon_brush(c, 20.0, 1.8, ACCENT),
            }
            text_center(&self.fonts.semi, label, r.x + 62.0, r.y + 76.0, 10.5, INK);
            let sub = match kind {
                0 => "10 ready to trace",
                1 => "Guided lessons",
                _ => "Fill with colour",
            };
            text_center(&self.fonts.regular, sub, r.x + 62.0, r.y + 92.0, 8.5, GREY);
            if self.button(r) {
                self.set_act(match kind {
                    0 => Act::Library,
                    1 => Act::StepOpen(1),
                    _ => Act::Open(7),
                });
            }
        }

        // featured
        text(&self.fonts.bold, "Featured today", 20.0, 528.0, 15.0, INK);
        for i in 0..2 {
            let idx = if i == 0 { 1 } else { 2 };
            let a = &self.arts[idx];
            let x = 18.0 + i as f32 * 204.0;
            let r = Rect::new(x, 540.0, 190.0, 166.0);
            shadow(r.x, r.y, r.w, r.h, 20.0, 0.8);
            rrect(r.x, r.y, r.w, r.h, 20.0, SNOW);
            rrect(r.x, r.y, r.w, 118.0, 20.0, rgba(255.0, 246.0, 249.0, 1.0));
            draw_art(
                a,
                vec2(r.x + 95.0, r.y + 62.0),
                104.0,
                0.0,
                0.9,
                rgba(88.0, 80.0, 94.0, 1.0),
                0.8,
                3,
            );
            text(&self.fonts.semi, a.name, r.x + 14.0, r.y + 140.0, 12.0, INK);
            let chip = a.difficulty;
            let cw = text_w(&self.fonts.semi, chip, 8.5) + 16.0;
            rrect(r.x + 14.0, r.y + 148.0, cw, 16.0, 8.0, ACCENT_SOFT);
            text(&self.fonts.semi, chip, r.x + 22.0, r.y + 159.5, 8.5, ACCENT);
            let arrow = vec2(r.x + 170.0, r.y + 142.0);
            circle_line(arrow, 12.0, 1.2, rgba(253.0, 79.0, 119.0, 0.7));
            icon_play(arrow + vec2(1.0, 0.0), 8.0, ACCENT);
            if self.button(r) {
                self.set_act(Act::Open(idx));
            }
        }

        // promo
        let promo = Rect::new(18.0, 722.0, 394.0, 78.0);
        rrect(promo.x, promo.y, promo.w, promo.h, 20.0, LILAC);
        draw_circle(promo.x + 352.0, promo.y + 20.0, 42.0, rgba(255.0, 255.0, 255.0, 0.6));
        text(&self.fonts.bold, "Coloring book", 36.0, 750.0, 14.0, rgba(72.0, 48.0, 104.0, 1.0));
        text(
            &self.fonts.regular,
            "Relax and fill line art with colour",
            36.0,
            766.0,
            9.5,
            rgba(108.0, 88.0, 132.0, 1.0),
        );
        rrect(296.0, 736.0, 52.0, 22.0, 11.0, ACCENT);
        text_center(&self.fonts.semi, "NEW", 322.0, 751.0, 9.0, SNOW);
        if self.button(promo) {
            self.set_act(Act::Open(8));
        }

        self.nav_bar();
    }

    fn draw_library(&mut self) {
        rect(0.0, 0.0, VW, VH, SNOW);
        // header
        let back = Rect::new(10.0, 52.0, 40.0, 40.0);
        icon_back(vec2(30.0, 72.0), 20.0, 2.0, INK);
        if self.button(back) {
            self.set_act(Act::Home);
        }
        text(&self.fonts.bold, "Choose a drawing", 54.0, 74.0, 19.0, INK);
        text(
            &self.fonts.regular,
            &format!("{} sketches  Â·  tap one to start tracing", self.arts.len()),
            54.0,
            91.0,
            9.5,
            GREY,
        );
        let search = Rect::new(380.0, 56.0, 34.0, 34.0);
        rrect(search.x, search.y, search.w, search.h, 12.0, ACCENT_SOFT);
        icon_search(vec2(397.0, 73.0), 18.0, 1.7, ACCENT);
        if self.button(search) {
            self.toast = Some(("Search is not in this build".into(), 1.6));
        }

        // filter chips
        let filters = ["All", "Easy", "Medium", "Hard"];
        let mut x = 18.0;
        for (i, f) in filters.iter().enumerate() {
            let w = text_w(&self.fonts.semi, f, 10.5) + 30.0;
            let r = Rect::new(x, 108.0, w, 30.0);
            let on = self.filter == i;
            rrect(r.x, r.y, r.w, r.h, 15.0, if on { ACCENT } else { rgba(246.0, 242.0, 246.0, 1.0) });
            text_center(
                &self.fonts.semi,
                f,
                r.x + r.w / 2.0,
                r.y + 20.0,
                10.5,
                if on { SNOW } else { INK_SOFT },
            );
            if self.button(r) {
                self.set_act(Act::Filter(i));
            }
            x += w + 8.0;
        }

        let list: Vec<usize> = self
            .arts
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                self.filter == 0
                    || (self.filter == 1 && a.difficulty == "Easy")
                    || (self.filter == 2 && a.difficulty == "Medium")
                    || (self.filter == 3 && a.difficulty == "Hard")
            })
            .map(|(i, _)| i)
            .collect();

        // grid with simple drag scroll
        let top = 150.0;
        let avail = 816.0 - top;
        let rows = (list.len() + 1) / 2;
        let content = rows as f32 * 234.0;
        if self.input.down
            && self.input.pos.y > top
            && self.input.pos.y < 816.0
            && !self.trace.sheet
            && self.input.pos.x > 0.0
        {
            // only scroll when the drag did not start on a card press
            if self.slider_drag.is_none() {
                self.lib_scroll -= self.input.delta.y;
            }
        }
        self.lib_scroll = self.lib_scroll.clamp(0.0, (content - avail).max(0.0));

        for (n, idx) in list.iter().enumerate() {
            let col = n % 2;
            let row = n / 2;
            let x = 18.0 + col as f32 * 204.0;
            let y = top + row as f32 * 234.0 - self.lib_scroll;
            if y > 816.0 || y + 220.0 < top {
                continue;
            }
            let r = Rect::new(x, y, 190.0, 220.0);
            let a = &self.arts[*idx];
            shadow(r.x, r.y, r.w, r.h, 18.0, 0.9);
            rrect(r.x, r.y, r.w, r.h, 18.0, SNOW);
            rrect(r.x, r.y, r.w, 160.0, 18.0, rgba(253.0, 246.0, 249.0, 1.0));
            draw_art(
                a,
                vec2(r.x + 95.0, r.y + 78.0),
                138.0,
                0.0,
                0.92,
                rgba(84.0, 76.0, 90.0, 1.0),
                0.85,
                3,
            );
            text(&self.fonts.semi, a.name, r.x + 14.0, r.y + 184.0, 12.0, INK);
            text(&self.fonts.regular, a.topic, r.x + 14.0, r.y + 199.0, 8.5, GREY);
            let chip = a.difficulty;
            let cw = text_w(&self.fonts.semi, chip, 8.0) + 14.0;
            rrect(r.x + 190.0 - cw - 12.0, r.y + 174.0, cw, 16.0, 8.0, ACCENT_SOFT);
            text_center(
                &self.fonts.semi,
                chip,
                r.x + 190.0 - cw / 2.0 - 12.0,
                r.y + 185.5,
                8.0,
                ACCENT,
            );
            if self.button(r) {
                self.set_act(Act::Open(*idx));
            }
        }

        // fade the scrolling grid into the tab bar
        for i in 0..10 {
            let t = i as f32 / 10.0;
            draw_rectangle(0.0, 776.0 + i as f32 * 4.0, VW, 4.0, rgba(255.0, 255.0, 255.0, t));
        }

        self.nav_bar();
    }

    fn draw_gallery(&mut self) {
        rect(0.0, 0.0, VW, VH, SNOW);
        text(&self.fonts.bold, "My drawings", 22.0, 74.0, 20.0, INK);
        text(
            &self.fonts.regular,
            "Finished sketches stay on this phone for this session",
            22.0,
            92.0,
            9.5,
            GREY,
        );

        if self.gallery.is_empty() {
            let card = Rect::new(34.0, 150.0, 362.0, 360.0);
            rrect(card.x, card.y, card.w, card.h, 24.0, rgba(253.0, 246.0, 249.0, 1.0));
            rrect_line(card.x, card.y, card.w, card.h, 24.0, 1.5, rgba(253.0, 79.0, 119.0, 0.25));
            draw_art(
                &self.arts[1],
                vec2(215.0, 268.0),
                150.0,
                0.0,
                0.5,
                rgba(253.0, 79.0, 119.0, 1.0),
                1.0,
                3,
            );
            text_center(&self.fonts.bold, "Nothing here yet", 215.0, 388.0, 17.0, INK);
            text_center(
                &self.fonts.regular,
                "Finish a sketch and it will appear here.",
                215.0,
                410.0,
                10.0,
                GREY,
            );
            let b = Rect::new(135.0, 436.0, 160.0, 42.0);
            rrect(b.x, b.y, b.w, b.h, 21.0, ACCENT);
            text_center(&self.fonts.semi, "Start drawing", 215.0, 462.0, 12.0, SNOW);
            if self.button(b) {
                self.set_act(Act::Library);
            }
        } else {
            for (i, item) in self.gallery.iter_mut().enumerate() {
                let col = i % 2;
                let row = i / 2;
                let x = 18.0 + col as f32 * 204.0;
                let y = 130.0 + row as f32 * 262.0;
                if y > 800.0 {
                    break;
                }
                let r = Rect::new(x, y, 190.0, 246.0);
                shadow(r.x, r.y, r.w, r.h, 18.0, 0.9);
                rrect(r.x, r.y, r.w, r.h, 18.0, SNOW);
                let thumb = Rect::new(r.x + 10.0, r.y + 10.0, r.w - 20.0, r.h - 52.0);
                rrect(thumb.x, thumb.y, thumb.w, thumb.h, 12.0, PAPER);
                draw_art(
                    &self.arts[item.art],
                    vec2(thumb.x + thumb.w / 2.0, thumb.y + thumb.h / 2.0),
                    150.0,
                    0.0,
                    0.22 + item.opacity * 0.25,
                    rgba(120.0, 112.0, 128.0, 1.0),
                    0.7,
                    3,
                );
                for s in &item.strokes {
                    let pts: Vec<Vec2> = s
                        .pts
                        .iter()
                        .map(|p| vec2(thumb.x + p.x * thumb.w, thumb.y + p.y * thumb.h))
                        .collect();
                    polyline(&pts, s.width * 0.55, fade(s.color, s.alpha));
                }
                let name = self.arts[item.art].name;
                text(&self.fonts.semi, name, r.x + 12.0, r.y + 224.0, 11.5, INK);
                let mins = item.ago / 60.0;
                let when = if mins < 1.0 {
                    "just now".to_string()
                } else {
                    format!("{:.0} min ago", mins)
                };
                text_right(&self.fonts.regular, &when, r.x + r.w - 12.0, r.y + 224.0, 8.5, GREY);
                item.ago += get_frame_time();
            }
        }

        self.nav_bar();
    }

    fn draw_settings(&mut self) {
        rect(0.0, 0.0, VW, VH, rgba(250.0, 247.0, 249.0, 1.0));
        text(&self.fonts.bold, "Settings", 22.0, 74.0, 20.0, INK);

        let card = Rect::new(18.0, 100.0, 394.0, 160.0);
        rrect(card.x, card.y, card.w, card.h, 20.0, SNOW);
        text(&self.fonts.semi, "Default picture opacity", 36.0, 130.0, 12.0, INK);
        text(
            &self.fonts.regular,
            "How strongly a new sketch sits over the paper",
            36.0,
            146.0,
            9.0,
            GREY,
        );
        let mut v = self.default_opacity;
        let track = Rect::new(36.0, 176.0, 300.0, 6.0);
        rrect(track.x, track.y, track.w, track.h, 3.0, HAIR);
        let kx = track.x + track.w * ((v - OPACITY_MIN) / (OPACITY_MAX - OPACITY_MIN));
        rrect(track.x, track.y, (kx - track.x).max(0.0), track.h, 3.0, ACCENT);
        draw_circle(kx, track.y + 3.0, 8.0, INK);
        draw_circle(kx, track.y + 3.0, 4.0, SNOW);
        text_right(&self.fonts.semi, &format!("{:.0}%", v * 100.0), 396.0, 182.0, 11.0, INK);
        self.slider(21, track, &mut v, OPACITY_MIN, OPACITY_MAX);
        self.default_opacity = v;

        let row = Rect::new(18.0, 276.0, 394.0, 62.0);
        rrect(row.x, row.y, row.w, row.h, 18.0, SNOW);
        icon_grid(vec2(46.0, 307.0), 20.0, 1.6, ACCENT);
        text(&self.fonts.semi, "Guide grid", 70.0, 303.0, 12.0, INK);
        text(&self.fonts.regular, "Thirds to help line up the paper", 70.0, 318.0, 8.5, GREY);
        let tg = Rect::new(342.0, 294.0, 48.0, 26.0);
        rrect(tg.x, tg.y, tg.w, tg.h, 13.0, if self.guide { ACCENT } else { HAIR });
        draw_circle(tg.x + if self.guide { 34.0 } else { 14.0 }, tg.y + 13.0, 10.0, SNOW);
        if self.button(tg) {
            self.set_act(Act::Guide);
        }

        let row2 = Rect::new(18.0, 348.0, 394.0, 62.0);
        rrect(row2.x, row2.y, row2.w, row2.h, 18.0, SNOW);
        icon_trash(vec2(46.0, 379.0), 20.0, 1.6, ACCENT);
        text(&self.fonts.semi, "Clear my drawings", 70.0, 375.0, 12.0, INK);
        text(
            &self.fonts.regular,
            &format!("{} saved in this session", self.gallery.len()),
            70.0,
            390.0,
            8.5,
            GREY,
        );
        let clr = Rect::new(330.0, 366.0, 60.0, 28.0);
        rrect(clr.x, clr.y, clr.w, clr.h, 14.0, ACCENT_SOFT);
        text_center(&self.fonts.semi, "Clear", 360.0, 385.0, 10.0, ACCENT);
        if self.button(clr) {
            self.set_act(Act::ResetGallery);
        }

        let about = Rect::new(18.0, 432.0, 394.0, 190.0);
        rrect(about.x, about.y, about.w, about.h, 20.0, SNOW);
        text(&self.fonts.semi, "About", 36.0, 462.0, 12.0, INK);
        let lines = [
            ("Version", "5.2.8 (834)"),
            ("Engine", "Rust + macroquad"),
            ("Sketches", "10 built-in line drawings"),
        ];
        for (i, (k, val)) in lines.iter().enumerate() {
            let y = 490.0 + i as f32 * 24.0;
            text(&self.fonts.regular, k, 36.0, y, 10.0, GREY);
            text_right(&self.fonts.semi, val, 394.0, y, 10.0, INK_SOFT);
        }
        let note = "The tracing surface is drawn on this desktop build; the browser page puts your phone camera behind the sheet when the browser allows it. Everything else mirrors the original's tracing screen.";
        let mut y = 566.0;
        for l in wrap(&self.fonts.regular, note, 9.0, 340.0) {
            text(&self.fonts.regular, &l, 36.0, y, 9.0, GREY);
            y += 13.0;
        }

        self.nav_bar();
    }

    // ------------------------------------------------------------- tracing

    fn draw_trace(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.draw_desk(self.anim as f64);
        #[cfg(target_arch = "wasm32")]
        {
            // The web page keeps a live camera video behind a transparent canvas.
        }

        if !self.trace.locked && self.trace.sheet {
            // drawing covers the picture; leave it visible underneath
        }

        // dim while a settings drawer is open, like the original
        if self.trace.panel != Panel::None {
            rect(0.0, 0.0, VW, VH, rgba(56.0, 32.0, 78.0, 0.22));
        }

        self.draw_overlay();

        if self.guide {
            for i in 1..3 {
                dashed(
                    vec2(30.0, 30.0 + i as f32 * (VH - 60.0) / 3.0),
                    vec2(VW - 30.0, 30.0 + i as f32 * (VH - 60.0) / 3.0),
                    6.0,
                    6.0,
                    1.0,
                    rgba(255.0, 255.0, 255.0, 0.35),
                );
                dashed(
                    vec2(30.0 + i as f32 * (VW - 60.0) / 3.0, 30.0),
                    vec2(30.0 + i as f32 * (VW - 60.0) / 3.0, VH - 30.0),
                    6.0,
                    6.0,
                    1.0,
                    rgba(255.0, 255.0, 255.0, 0.35),
                );
            }
        }

        self.draw_strokes();

        // HUD reset each frame
        self.hud = Hud::default();

        if self.trace.hide_ui {
            let pill = Rect::new(330.0, 46.0, 86.0, 38.0);
            rrect(pill.x, pill.y, pill.w, pill.h, 19.0, rgba(255.0, 255.0, 255.0, 0.95));
            icon_eye(vec2(pill.x + 22.0, pill.y + 19.0), 16.0, 1.5, INK_SOFT, true);
            text(&self.fonts.semi, "Controls", pill.x + 36.0, pill.y + 24.0, 9.5, INK_SOFT);
            if self.button(pill) {
                self.set_act(Act::ToggleHide);
            }
            if self.modal == Modal::Finish {
                self.draw_finish_modal();
            }
            return;
        }

        // ---- top bar
        let bar = Rect::new(14.0, 46.0, 402.0, 46.0);
        shadow(bar.x, bar.y, bar.w, bar.h, 23.0, 0.7);
        rrect(bar.x, bar.y, bar.w, bar.h, 23.0, rgba(255.0, 255.0, 255.0, 0.97));
        let back = Rect::new(14.0, 46.0, 52.0, 46.0);
        icon_back(vec2(36.0, 69.0), 19.0, 2.0, INK);
        if self.button(back) {
            self.set_act(Act::Back);
        }
        rrect(72.0, 59.0, 20.0, 20.0, 6.0, ACCENT);
        rrect(76.0, 63.0, 12.0, 12.0, 4.0, SNOW);
        text(&self.fonts.semi, "AR Drawing", 98.0, 74.0, 13.5, INK);
        self.hud.title = true;

        let finish = Rect::new(332.0, 53.0, 74.0, 32.0);
        rrect(finish.x, finish.y, finish.w, finish.h, 16.0, ACCENT_SOFT);
        text_center(&self.fonts.semi, "Finish", finish.x + finish.w / 2.0, finish.y + 21.0, 11.0, ACCENT);
        if self.button(finish) {
            self.set_act(Act::AskFinish);
        }
        self.hud.finish = true;

        // right side: hide the controls for a clean tracing view
        let eye = Rect::new(292.0, 53.0, 30.0, 32.0);
        icon_eye(vec2(307.0, 69.0), 17.0, 1.5, INK_SOFT, true);
        if self.button(eye) {
            self.set_act(Act::ToggleHide);
        }

        // ---- step pill
        if self.trace.step_mode {
            let pill = Rect::new(16.0, 602.0, 128.0, 38.0);
            shadow(pill.x, pill.y, pill.w, pill.h, 19.0, 0.6);
            rrect(pill.x, pill.y, pill.w, pill.h, 19.0, rgba(255.0, 255.0, 255.0, 0.96));
            let l = Rect::new(pill.x + 4.0, pill.y + 4.0, 30.0, 30.0);
            icon_back(vec2(l.x + 16.0, l.y + 15.0), 12.0, 1.8, if self.trace.step > 1 { INK } else { GREY_LIGHT });
            if self.button(l) {
                self.set_act(Act::PrevStep);
            }
            text_center(
                &self.fonts.semi,
                &format!("{} / 4", self.trace.step),
                pill.x + 64.0,
                pill.y + 24.0,
                11.0,
                INK,
            );
            let rside = Rect::new(pill.x + 96.0, pill.y + 4.0, 30.0, 30.0);
            let cx = rside.x + 15.0;
            let cy = rside.y + 15.0;
            let col = if self.trace.step < 4 { ACCENT } else { GREY_LIGHT };
            draw_line(cx - 3.0, cy - 6.0, cx + 3.0, cy, 1.8, col);
            draw_line(cx + 3.0, cy, cx - 3.0, cy + 6.0, 1.8, col);
            if self.button(rside) {
                self.set_act(Act::NextStep);
            }
        }

        // ---- bottom sheet: drawing tools or the settings panel
        if self.trace.sheet {
            self.draw_tool_sheet();
        } else {
            let panel = Rect::new(0.0, 726.0, VW, 174.0);
            shadow(panel.x, panel.y, panel.w, panel.h, 26.0, 0.8);
            rrect(panel.x, panel.y, panel.w, panel.h, 26.0, rgba(255.0, 255.0, 255.0, 0.98));

            if self.trace.panel == Panel::Scaling {
                text(&self.fonts.semi, "Scaling", 34.0, 748.0, 12.5, INK);
                text(
                    &self.fonts.regular,
                    "Adjust zoom level and scale of the camera",
                    34.0,
                    760.0,
                    8.5,
                    GREY,
                );
                let mut sv = self.trace.scale;
                let track = Rect::new(34.0, 773.0, 300.0, 5.0);
                rrect(track.x, track.y, track.w, track.h, 2.5, HAIR);
                let kx = track.x + track.w * ((sv - SCALE_MIN) / (SCALE_MAX - SCALE_MIN));
                rrect(track.x, track.y, (kx - track.x).max(0.0), track.h, 2.5, ACCENT);
                draw_circle(kx, track.y + 2.5, 8.0, INK);
                draw_circle(kx, track.y + 2.5, 4.0, SNOW);
                text_right(&self.fonts.semi, &format!("{:.1}x", sv), 400.0, 779.0, 11.0, INK);
                self.slider(11, track, &mut sv, SCALE_MIN, SCALE_MAX);
                self.trace.scale = sv;

                // The app's own zoom labels are "0.5x" (0x8aacdc), "1.0x"
                // (0x865d24) and "2.0x" (0x721604); ZoomSettingsItem (0x128b808)
                // renders these as a preset row.
                let centre = Rect::new(34.0, 782.0, 92.0, 22.0);
                rrect(centre.x, centre.y, centre.w, centre.h, 11.0, ACCENT_SOFT);
                text_center(&self.fonts.semi, "Center", centre.x + 46.0, centre.y + 15.0, 9.0, ACCENT);
                if self.button(centre) {
                    self.set_act(Act::Center);
                }
                for (i, preset) in SCALE_PRESETS.iter().enumerate() {
                    let r = Rect::new(136.0 + i as f32 * 58.0, 782.0, 54.0, 22.0);
                    let on = (self.trace.scale - preset).abs() < 0.01;
                    rrect(r.x, r.y, r.w, r.h, 11.0, if on { ACCENT } else { rgba(246.0, 243.0, 246.0, 1.0) });
                    text_center(
                        &self.fonts.semi,
                        &format!("{:.1}x", preset),
                        r.x + 27.0,
                        r.y + 15.0,
                        9.0,
                        if on { SNOW } else { INK_SOFT },
                    );
                    if self.button(r) {
                        self.set_act(Act::ScalePreset(*preset));
                    }
                }
            } else {
                text(&self.fonts.semi, "Opacity Adjustment", 34.0, 756.0, 12.5, INK);
                text(
                    &self.fonts.regular,
                    "Here you can change the opacity of the image",
                    34.0,
                    770.0,
                    7.8,
                    GREY,
                );
                let mut ov = self.trace.opacity;
                // _OpacityButtonState.build (0xfbe3d4) offers a 0.5/1.0 quick
                // value; it is the app's fastest way to change transparency.
                let quick = Rect::new(336.0, 748.0, 62.0, 24.0);
                rrect(quick.x, quick.y, quick.w, quick.h, 12.0, ACCENT_SOFT);
                text_center(
                    &self.fonts.semi,
                    &format!("{:.0}%", ov * 100.0),
                    quick.x + 31.0,
                    quick.y + 16.0,
                    9.0,
                    ACCENT,
                );
                if self.button(quick) {
                    self.set_act(Act::OpacityToggle);
                }
                let track = Rect::new(34.0, 794.0, 300.0, 5.0);
                rrect(track.x, track.y, track.w, track.h, 2.5, HAIR);
                // Real formula from _OpacitySettingsState's pan closures
                // (0x100a8c8 / 0x100a9b8): opacity = clamp(x / trackWidth, 0, 1).
                let kx = track.x
                    + track.w * ((ov - OPACITY_MIN) / (OPACITY_MAX - OPACITY_MIN)).clamp(0.0, 1.0);
                rrect(track.x, track.y, (kx - track.x).max(0.0), track.h, 2.5, ACCENT);
                draw_circle(kx, track.y + 2.5, 8.5, INK);
                draw_circle(kx, track.y + 2.5, 4.5, SNOW);
                text_right(&self.fonts.semi, &format!("{:.0}%", ov * 100.0), 400.0, 801.0, 11.0, INK);
                let before = ov;
                self.slider(10, track, &mut ov, OPACITY_MIN, OPACITY_MAX);
                self.trace.opacity = ov;
                if (ov - before).abs() > 0.0001 {
                    self.trace.opacity_settle = OPACITY_SETTLE_SECS;
                }
                self.hud.opacity = true;

                // lock button at the right of the slider row
                let lock = Rect::new(378.0, 782.0, 30.0, 30.0);
                rrect(lock.x, lock.y, lock.w, lock.h, 10.0, if self.trace.locked { ACCENT_SOFT } else { rgba(246.0, 243.0, 246.0, 1.0) });
                icon_lock(
                    vec2(lock.x + 15.0, lock.y + 16.0),
                    16.0,
                    1.6,
                    if self.trace.locked { ACCENT } else { GREY },
                    !self.trace.locked,
                );
                if self.button(lock) {
                    self.set_act(Act::ToggleLock);
                }
            }

            // tool row
            // Labels are the original's own strings: "Opacity", "Scaling",
            // "Center" (0x690864) and "Flashlight" (0x89e4fc) in libapp.so.
            let labels = ["Opacity", "Scaling", "Center", "Flashlight"];
            for i in 0..4 {
                let cx = 62.0 + i as f32 * 102.0;
                let r = Rect::new(cx - 46.0, 808.0, 92.0, 78.0);
                let active = (i == 0 && self.trace.panel == Panel::Opacity)
                    || (i == 1 && self.trace.panel == Panel::Scaling)
                    || (i == 3 && self.trace.flashlight);
                if active {
                    rrect(cx - 24.0, 812.0, 48.0, 40.0, 14.0, ACCENT_SOFT);
                }
                let c = vec2(cx, 832.0);
                let col = if active { ACCENT } else { INK_SOFT };
                match i {
                    0 => icon_eye(c, 20.0, 1.6, col, true),
                    1 => icon_search(c, 20.0, 1.6, col),
                    2 => icon_target(c, 20.0, 1.5, col),
                    _ => icon_flash(c, 20.0, col),
                }
                text_center(&self.fonts.regular, labels[i], cx, 868.0, 8.5, if active { ACCENT } else { GREY });
                if self.button(r) {
                    self.set_act(match i {
                        0 => Act::Panel(Panel::Opacity),
                        1 => Act::Panel(Panel::Scaling),
                        2 => Act::Center,
                        _ => Act::ToggleFlash,
                    });
                }
            }
            self.hud.tools = true;
        }

        if self.trace.flashlight {
            rect(0.0, 0.0, VW, VH, rgba(255.0, 247.0, 230.0, 0.06));
        }

        // pencil FAB
        if !self.trace.sheet {
            let fab = Rect::new(358.0, 656.0, 56.0, 56.0);
            shadow(fab.x, fab.y, fab.w, fab.h, 28.0, 1.0);
            rrect(fab.x, fab.y, fab.w, fab.h, 28.0, ACCENT);
            icon_pencil(vec2(fab.x + 28.0, fab.y + 28.0), 26.0, 2.0, SNOW);
            if self.button(fab) {
                self.set_act(Act::ToggleSheet);
            }
        }

        self.trace_input();

        if self.modal == Modal::Finish {
            self.draw_finish_modal();
        }
    }

    fn draw_desk(&self, t: f64) {
        let sway = (t * 0.9).sin() as f32 * 0.8;
        vgrad(0.0, 0.0, VW, VH, rgba(203.0, 170.0, 129.0, 1.0), rgba(140.0, 104.0, 70.0, 1.0));
        // grain
        for i in 0..46 {
            let y = i as f32 * 20.0 + ((i * 37) % 11) as f32;
            let a = 0.03 + ((i * 13) % 7) as f32 * 0.006;
            draw_line(0.0, y, VW, y + ((i * 19) % 9) as f32 - 4.0, 1.0, rgba(92.0, 63.0, 40.0, a));
            draw_line(0.0, y + 2.0, VW, y + ((i * 23) % 7) as f32 - 3.0, 1.0, rgba(255.0, 228.0, 193.0, a * 0.7));
        }
        for i in 0..5 {
            let x = 40.0 + i as f32 * 92.0;
            draw_line(x, 0.0, x + 3.0, VH, 1.0, rgba(76.0, 52.0, 33.0, 0.10));
        }
        // paper
        let p = Rect::new(26.0 + sway, 74.0, 378.0, 792.0);
        for i in 0..5 {
            let k = i as f32;
            rrect(
                p.x - k * 2.0,
                p.y + 4.0 + k * 2.0,
                p.w + k * 4.0,
                p.h + k * 2.0,
                6.0,
                rgba(40.0, 26.0, 16.0, 0.05 - k * 0.008),
            );
        }
        rrect(p.x, p.y, p.w, p.h, 4.0, rgba(250.0, 248.0, 242.0, 1.0));
        // paper fibre: a light tooth so the sheet does not read as a flat fill
        for i in 0..90 {
            let fx = p.x + 8.0 + ((i * 67) % 360) as f32;
            let fy = p.y + 6.0 + ((i * 131) % 770) as f32;
            let len = 3.0 + ((i * 7) % 9) as f32;
            draw_line(fx, fy, fx + len, fy + ((i % 3) as f32 - 1.0), 1.0, rgba(120.0, 110.0, 92.0, 0.035));
        }
        vgrad(p.x, p.y, p.w, 60.0, rgba(255.0, 253.0, 248.0, 1.0), rgba(250.0, 248.0, 242.0, 0.0));
        draw_line(p.x, p.y, p.x + p.w, p.y, 1.0, rgba(0.0, 0.0, 0.0, 0.05));
        draw_line(p.x, p.y + p.h, p.x + p.w, p.y + p.h, 1.0, rgba(0.0, 0.0, 0.0, 0.04));
        // soft light from above
        vgrad(0.0, 0.0, VW, 180.0, rgba(255.0, 250.0, 235.0, 0.12), rgba(255.0, 250.0, 235.0, 0.0));
    }

    fn overlay_rect(&self) -> Rect {
        let w = 320.0 * self.trace.scale;
        let h = 358.0 * self.trace.scale;
        Rect::new(self.trace.pos.x - w / 2.0, self.trace.pos.y - h / 2.0, w, h)
    }

    fn draw_overlay(&mut self) {
        let tr_pos = self.trace.pos;
        let s = self.trace.scale;
        let rot = self.trace.rot;
        let op = self.trace.opacity;
        let w = 320.0 * s;
        let h = 358.0 * s;

        let hw = w / 2.0;
        let hh = h / 2.0;
        let rr = 14.0 * s;

        // rounded sheet as one polygon, so a translucent sheet has no darker seams
        let mut ring: Vec<Vec2> = Vec::new();
        let specs = [
            (hw - rr, -hh + rr, -std::f32::consts::FRAC_PI_2),
            (hw - rr, hh - rr, 0.0),
            (-hw + rr, hh - rr, std::f32::consts::FRAC_PI_2),
            (-hw + rr, -hh + rr, std::f32::consts::PI),
        ];
        for (cx, cy, a0) in specs {
            for i in 0..=6 {
                let a = a0 + std::f32::consts::FRAC_PI_2 * (i as f32 / 6.0);
                let local = vec2(cx + rr * a.cos(), cy + rr * a.sin());
                ring.push(tr_pos
                    + vec2(
                        local.x * rot.cos() - local.y * rot.sin(),
                        local.x * rot.sin() + local.y * rot.cos(),
                    ));
            }
        }

        let card = rgba(255.0, 253.0, 251.0, 0.30 + op * 0.70);
        for i in 0..ring.len() {
            draw_triangle(tr_pos, ring[i], ring[(i + 1) % ring.len()], card);
        }
        // rim
        for i in 0..ring.len() {
            draw_line(
                ring[i].x,
                ring[i].y,
                ring[(i + 1) % ring.len()].x,
                ring[(i + 1) % ring.len()].y,
                1.1,
                rgba(90.0, 80.0, 90.0, op * 0.20),
            );
        }

        draw_art(
            &self.arts[self.trace.art],
            tr_pos,
            (w * 0.88).min(h * 0.86),
            rot,
            op * 0.95,
            rgba(62.0, 56.0, 66.0, 1.0),
            s,
            self.trace.step,
        );

        // tiny brand line on the card, like the original's watermark
        let water = vec2(-w / 2.0 + 18.0 * s, h / 2.0 - 16.0 * s);
        let wp = tr_pos
            + vec2(
                water.x * rot.cos() - water.y * rot.sin(),
                water.x * rot.sin() + water.y * rot.cos(),
            );
        text(
            &self.fonts.semi,
            "AR Drawing",
            wp.x,
            wp.y,
            8.0 * s,
            rgba(120.0, 110.0, 120.0, op * 0.55),
        );
    }

    fn draw_strokes(&mut self) {
        for stroke in &self.trace.strokes {
            self.draw_stroke(stroke);
        }
        if let Some(st) = &self.trace.current {
            self.draw_stroke(st);
        }
    }

    fn draw_stroke(&self, s: &Stroke) {
        if s.pts.len() < 2 {
            if let Some(p) = s.pts.first() {
                draw_circle(p.x, p.y, s.width / 2.0, fade(s.color, s.alpha));
            }
            return;
        }
        let jitter = if s.tool == Tool::Pencil { 0.0 } else { 0.0 };
        // smooth the polyline through midpoints
        for i in 0..s.pts.len() - 1 {
            let a = s.pts[i];
            let b = s.pts[i + 1];
            let mid_a = if i == 0 { a } else { (s.pts[i - 1] + a) / 2.0 };
            let mid_b = if i == 1 { a } else { (a + b) / 2.0 };
            let _ = mid_a;
            let _ = mid_b;
            draw_line(a.x + jitter, a.y, b.x, b.y, s.width, fade(s.color, s.alpha));
        }
        if s.tool == Tool::Brush || s.tool == Tool::Marker {
            for i in 0..s.pts.len() - 1 {
                let a = s.pts[i];
                let b = s.pts[i + 1];
                draw_circle(a.x, a.y, s.width / 2.0, fade(s.color, s.alpha));
                draw_circle(b.x, b.y, s.width / 2.0, fade(s.color, s.alpha));
            }
        }
        if s.tool == Tool::Marker {
            // translucent second pass to fake the felt-tip edge
            for i in 0..s.pts.len() - 1 {
                let a = s.pts[i];
                let b = s.pts[i + 1];
                draw_line(a.x, a.y, b.x, b.y, s.width * 0.55, fade(SNOW, 0.10));
            }
        }
    }

    fn draw_tool_sheet(&mut self) {
        let sheet = Rect::new(0.0, SHEET_Y, VW, VH - SHEET_Y);
        shadow(sheet.x, sheet.y, sheet.w, sheet.h, 26.0, 0.9);
        rrect(sheet.x, sheet.y, sheet.w, sheet.h, 26.0, rgba(255.0, 255.0, 255.0, 0.99));
        rrect(195.0, 610.0, 40.0, 4.0, 2.0, HAIR);

        text(&self.fonts.bold, "Draw", 20.0, 642.0, 15.0, INK);
        let close = Rect::new(392.0, 618.0, 30.0, 30.0);
        icon_close(vec2(407.0, 633.0), 14.0, 1.8, INK);
        if self.button(close) {
            self.set_act(Act::CloseSheet);
        }

        // tools
        let tools = [Tool::Pencil, Tool::Marker, Tool::Brush, Tool::Eraser];
        for (i, t) in tools.iter().enumerate() {
            let x = 16.0 + i as f32 * 101.0;
            let r = Rect::new(x, 654.0, 94.0, 34.0);
            let on = self.trace.tool == *t;
            rrect(r.x, r.y, r.w, r.h, 17.0, if on { ACCENT } else { rgba(246.0, 242.0, 246.0, 1.0) });
            let col = if on { SNOW } else { INK_SOFT };
            let c = vec2(r.x + 22.0, r.y + 17.0);
            match t {
                Tool::Pencil => icon_pencil(c, 18.0, 1.7, col),
                Tool::Marker => icon_marker(c, 18.0, 1.7, col),
                Tool::Brush => icon_brush(c, 18.0, 1.7, col),
                Tool::Eraser => icon_eraser(c, 18.0, 1.7, col),
            }
            text(&self.fonts.semi, t.name(), r.x + 36.0, r.y + 21.0, 9.5, col);
            if self.button(r) {
                self.set_act(Act::Tool(*t));
            }
        }

        // colours
        for i in 0..10 {
            let c = vec2(26.0 + i as f32 * 38.0, 706.0);
            if self.trace.color == i {
                circle_line(c, 14.0, 2.0, ACCENT);
            }
            draw_circle(c.x, c.y, 11.0, STROKE_COLORS[i]);
        }
        for i in 0..10 {
            let c = vec2(26.0 + i as f32 * 38.0, 706.0);
            if self.input.pressed && (self.input.pos - c).length() < 17.0 {
                self.set_act(Act::Color(i));
            }
        }

        // size
        text(&self.fonts.semi, "Size", 20.0, 752.0, 10.0, INK_SOFT);
        let mut sz = self.trace.size;
        let track = Rect::new(58.0, 748.0, 268.0, 5.0);
        rrect(track.x, track.y, track.w, track.h, 2.5, HAIR);
        let kx = track.x + track.w * ((sz - SIZE_MIN) / (SIZE_MAX - SIZE_MIN));
        rrect(track.x, track.y, (kx - track.x).max(0.0), track.h, 2.5, ACCENT);
        draw_circle(kx, track.y + 2.5, 8.0, INK);
        draw_circle(kx, track.y + 2.5, 4.0, SNOW);
        self.slider(12, track, &mut sz, SIZE_MIN, SIZE_MAX);
        self.trace.size = sz;
        draw_circle(360.0, 748.0, (sz * 0.5).clamp(1.0, 10.0), STROKE_COLORS[self.trace.color]);
        text_right(
            &self.fonts.semi,
            &format!("{:.0} · {:.2}", sz, self.trace.pencil_coefficient),
            404.0,
            757.0,
            9.0,
            INK,
        );

        // actions
        let undo = Rect::new(16.0, 772.0, 124.0, 32.0);
        let redo = Rect::new(153.0, 772.0, 124.0, 32.0);
        let clear = Rect::new(290.0, 772.0, 124.0, 32.0);
        for (r, label, kind) in [
            (undo, "Undo", 0),
            (redo, "Redo", 1),
            (clear, "Clear", 2),
        ] {
            rrect(r.x, r.y, r.w, r.h, 16.0, rgba(247.0, 243.0, 247.0, 1.0));
            let c = vec2(r.x + 28.0, r.y + 16.0);
            match kind {
                0 => icon_undo(c, 16.0, 1.6, INK_SOFT, false),
                1 => icon_undo(c, 16.0, 1.6, INK_SOFT, true),
                _ => icon_trash(c, 16.0, 1.6, INK_SOFT),
            }
            text(&self.fonts.semi, label, r.x + 44.0, r.y + 20.0, 9.5, INK_SOFT);
            if self.button(r) {
                self.set_act(match kind {
                    0 => Act::Undo,
                    1 => Act::Redo,
                    _ => Act::Clear,
                });
            }
        }
        text_center(
            &self.fonts.regular,
            "Your drawing stays on the screen; Finish saves it to My drawings.",
            215.0,
            828.0,
            8.0,
            GREY,
        );
    }

    fn trace_input(&mut self) {
        // pinch
        if let Some(second) = self.input.second {
            let d = (self.input.pos - second).length();
            let a = (second - self.input.pos).y.atan2((second - self.input.pos).x);
            if let Some((pd, pa, _)) = self.trace.pinch {
                let factor = (d / pd.max(1.0)).clamp(0.9, 1.1);
                self.trace.scale = (self.trace.scale * factor).clamp(SCALE_MIN, SCALE_MAX);
                self.trace.rot += (a - pa) * 0.5;
                self.trace.panel = Panel::None;
            } else {
                self.trace.pinch = Some((d, a, self.trace.pos));
            }
            return;
        } else {
            self.trace.pinch = None;
        }

        if self.modal != Modal::None {
            return;
        }

        // UI blockers: top bar, panel, sheet, fab
        let ui_rects = if self.trace.hide_ui {
            vec![Rect::new(330.0, 46.0, 86.0, 38.0)]
        } else if self.trace.sheet {
            vec![
                Rect::new(0.0, SHEET_Y, VW, VH - SHEET_Y),
                Rect::new(0.0, 0.0, VW, 100.0),
            ]
        } else {
            vec![
                Rect::new(0.0, 0.0, VW, 100.0),
                Rect::new(0.0, 726.0, VW, VH - 726.0),
                Rect::new(352.0, 648.0, 68.0, 70.0),
            ]
        };
        let on_ui = ui_rects.iter().any(|r| r.contains(self.input.pos));

        if self.trace.sheet && !on_ui {
            if self.input.pressed {
                self.trace.push_undo();
                let col = STROKE_COLORS[self.trace.color];
                let (width, alpha) = match self.trace.tool {
                    Tool::Pencil => (self.trace.size, 1.0),
                    Tool::Marker => (self.trace.size * 1.8, 0.55),
                    Tool::Brush => (self.trace.size * 1.35, 0.9),
                    Tool::Eraser => (self.trace.size * 2.4, 1.0),
                };
                self.trace.current = Some(Stroke {
                    pts: vec![self.input.pos],
                    color: col,
                    width,
                    alpha,
                    tool: self.trace.tool,
                });
            } else if self.input.down {
                if let Some(cur) = self.trace.current.as_mut() {
                    let p = self.input.pos;
                    let last = *cur.pts.last().unwrap();
                    if (p - last).length() > 1.2 {
                        cur.pts.push(p);
                    }
                }
            }
            if self.input.released {
                if let Some(cur) = self.trace.current.take() {
                    if cur.tool == Tool::Eraser {
                        let r = self.trace.size * 2.4;
                        let mut changed = false;
                        for p in &cur.pts {
                            changed |= erase_at(&mut self.trace.strokes, *p, r);
                        }
                        if !changed {
                            self.trace.undo_stack.pop();
                        }
                    } else if cur.pts.len() > 1 {
                        self.trace.strokes.push(cur);
                    }
                }
            }
            return;
        }

        // overlay drag
        if !self.trace.sheet && !self.trace.locked {
            let over = Rect::new(
                self.trace.pos.x - 160.0 * self.trace.scale,
                self.trace.pos.y - 179.0 * self.trace.scale,
                320.0 * self.trace.scale,
                358.0 * self.trace.scale,
            );
            if self.input.pressed && over.contains(self.input.pos) && !on_ui {
                self.trace.dragging = true;
                self.trace.drag_off = self.trace.pos - self.input.pos;
            }
            if !self.input.down {
                self.trace.dragging = false;
            }
            if self.trace.dragging && self.input.down {
                self.trace.pos = self.input.pos + self.trace.drag_off;
            }
        }

        if self.input.wheel.abs() > 0.01 && !on_ui {
            self.trace.scale =
                (self.trace.scale * (1.0 + self.input.wheel * 0.0015)).clamp(SCALE_MIN, SCALE_MAX);
        }
    }

    fn draw_finish_modal(&mut self) {
        rect(0.0, 0.0, VW, VH, rgba(20.0, 12.0, 24.0, 0.42));
        let m = Rect::new(38.0, 220.0, 354.0, 400.0);
        shadow(m.x, m.y, m.w, m.h, 26.0, 1.2);
        rrect(m.x, m.y, m.w, m.h, 26.0, SNOW);
        rrect(140.0, 236.0, 150.0, 150.0, 18.0, rgba(253.0, 246.0, 249.0, 1.0));
        draw_art(
            &self.arts[self.trace.art],
            vec2(215.0, 311.0),
            122.0,
            0.0,
            0.95,
            rgba(74.0, 66.0, 80.0, 1.0),
            0.9,
            3,
        );
        // the strokes, scaled into the preview
        for s in &self.trace.strokes {
            let pts: Vec<Vec2> = s
                .pts
                .iter()
                .map(|p| {
                    vec2(
                        140.0 + (p.x / VW) * 150.0,
                        236.0 + ((p.y - PHOTO_TOP) / PHOTO_H) * 150.0,
                    )
                })
                .collect();
            polyline(&pts, s.width * 0.5, fade(s.color, s.alpha));
        }
        text_center(&self.fonts.bold, "Finish this drawing?", 215.0, 414.0, 17.0, INK);
        let mins = ((self.time - self.trace.started) / 60.0).max(0.0);
        let detail = if mins < 1.0 {
            "After completing your drawing, press Finish.".to_string()
        } else {
            format!("After completing your drawing, press Finish.  Â·  {:.0} min", mins)
        };
        text_center(&self.fonts.regular, &detail, 215.0, 436.0, 9.5, GREY);

        let keep = Rect::new(60.0, 470.0, 144.0, 44.0);
        rrect(keep.x, keep.y, keep.w, keep.h, 22.0, rgba(247.0, 243.0, 247.0, 1.0));
        text_center(&self.fonts.semi, "Keep drawing", keep.x + 72.0, keep.y + 28.0, 11.5, INK_SOFT);
        if self.button(keep) {
            self.set_act(Act::KeepDrawing);
        }
        let save = Rect::new(226.0, 470.0, 144.0, 44.0);
        rrect(save.x, save.y, save.w, save.h, 22.0, ACCENT);
        text_center(&self.fonts.semi, "Finish", save.x + 72.0, save.y + 28.0, 11.5, SNOW);
        if self.button(save) {
            self.set_act(Act::Save);
        }

        let strokes = self.trace.strokes.len();
        text_center(
            &self.fonts.regular,
            &format!("{} stroke(s)  Â·  {:.0}% opacity", strokes, self.trace.opacity * 100.0),
            215.0,
            546.0,
            9.0,
            GREY,
        );
    }
}

fn yes(b: bool) -> &'static str {
    if b {
        "drawn"
    } else {
        "missing"
    }
}

fn erase_at(strokes: &mut Vec<Stroke>, p: Vec2, r: f32) -> bool {
    let mut changed = false;
    let mut out: Vec<Stroke> = Vec::new();
    for s in strokes.drain(..) {
        let mut cur: Vec<Vec2> = Vec::new();
        let mut pieces: Vec<Vec<Vec2>> = Vec::new();
        for pt in s.pts.iter() {
            if (*pt - p).length() < r {
                changed = true;
                if cur.len() > 1 {
                    pieces.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
            } else {
                cur.push(*pt);
            }
        }
        if cur.len() > 1 {
            pieces.push(cur);
        }
        for piece in pieces {
            out.push(Stroke { pts: piece, ..s.clone() });
        }
    }
    *strokes = out;
    changed
}

