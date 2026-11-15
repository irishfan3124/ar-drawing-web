// Drawing kit: the shapes, type and icons the app is built from.
// Layout is done in a 430 x 900 "phone" space and scaled to the window.

use macroquad::prelude::*;

pub const VW: f32 = 430.0;
pub const VH: f32 = 900.0;

pub const ACCENT: Color = Color::new(0.992, 0.310, 0.467, 1.0);
pub const ACCENT_DARK: Color = Color::new(0.878, 0.204, 0.372, 1.0);
pub const ACCENT_SOFT: Color = Color::new(1.0, 0.910, 0.937, 1.0);
pub const LILAC: Color = Color::new(0.945, 0.910, 0.996, 1.0);
pub const INK: Color = Color::new(0.090, 0.086, 0.102, 1.0);
pub const INK_SOFT: Color = Color::new(0.294, 0.286, 0.314, 1.0);
pub const GREY: Color = Color::new(0.420, 0.408, 0.447, 1.0);
pub const GREY_LIGHT: Color = Color::new(0.827, 0.820, 0.843, 1.0);
pub const HAIR: Color = Color::new(0.902, 0.894, 0.914, 1.0);
pub const PAPER: Color = Color::new(1.0, 0.996, 0.988, 1.0);
pub const SNOW: Color = Color::new(1.0, 1.0, 1.0, 1.0);
pub const LINE: Color = Color::new(0.545, 0.514, 0.553, 1.0);

pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r / 255.0, g / 255.0, b / 255.0, a)
}

pub fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

pub struct Fonts {
    pub regular: Font,
    pub semi: Font,
    pub bold: Font,
}

impl Fonts {
    pub fn load() -> Self {
        let regular = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Poppins-Regular.ttf"))
            .expect("poppins regular");
        let semi = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Poppins-SemiBold.ttf"))
            .expect("poppins semibold");
        let bold = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Poppins-Bold.ttf"))
            .expect("poppins bold");
        Self { regular, semi, bold }
    }
}

// ---------------------------------------------------------------- primitives

pub fn rect(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_rectangle(x, y, w, h, c);
}

pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    if r <= 0.5 {
        draw_rectangle(x, y, w, h, c);
        return;
    }
    // One triangle fan: no overlapping pieces, so translucent fills stay even.
    let mut pts: Vec<Vec2> = Vec::new();
    let centers = [
        (x + w - r, y + r, -std::f32::consts::FRAC_PI_2),
        (x + w - r, y + h - r, 0.0),
        (x + r, y + h - r, std::f32::consts::FRAC_PI_2),
        (x + r, y + r, std::f32::consts::PI),
    ];
    let n = 5;
    for (cx, cy, a0) in centers {
        for i in 0..=n {
            let a = a0 + std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
            pts.push(vec2(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    let mid = vec2(x + w / 2.0, y + h / 2.0);
    for i in 0..pts.len() {
        draw_triangle(mid, pts[i], pts[(i + 1) % pts.len()], c);
    }
}

pub fn rrect_line(x: f32, y: f32, w: f32, h: f32, r: f32, t: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let n = 8;
    let mut ring: Vec<Vec2> = Vec::new();
    // corners: (cx, cy, start angle)
    let corners = [
        (x + w - r, y + r, -std::f32::consts::FRAC_PI_2),
        (x + w - r, y + h - r, 0.0),
        (x + r, y + h - r, std::f32::consts::FRAC_PI_2),
        (x + r, y + r, std::f32::consts::PI),
    ];
    for (cx, cy, a0) in corners {
        for i in 0..=n {
            let a = a0 + std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
            ring.push(vec2(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    for i in 0..ring.len() {
        let a = ring[i];
        let b = ring[(i + 1) % ring.len()];
        draw_line(a.x, a.y, b.x, b.y, t, c);
    }
}

pub fn shadow(x: f32, y: f32, w: f32, h: f32, r: f32, strength: f32) {
    for i in 0..5 {
        let k = i as f32;
        let spread = k * 2.0 + 1.0;
        let a = strength * (0.045 - k * 0.0075).max(0.0);
        rrect(
            x - spread,
            y - spread + 2.0,
            w + spread * 2.0,
            h + spread * 2.0,
            r + spread,
            Color::new(0.09, 0.07, 0.12, a),
        );
    }
}

pub fn vgrad(x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) {
    let steps = 40;
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let c = Color::new(
            top.r + (bottom.r - top.r) * t,
            top.g + (bottom.g - top.g) * t,
            top.b + (bottom.b - top.b) * t,
            top.a + (bottom.a - top.a) * t,
        );
        draw_rectangle(x, y + h * i as f32 / steps as f32, w, h / steps as f32 + 1.0, c);
    }
}

pub fn line(a: Vec2, b: Vec2, t: f32, c: Color) {
    draw_line(a.x, a.y, b.x, b.y, t, c);
}

pub fn polyline(pts: &[Vec2], t: f32, c: Color) {
    for w in pts.windows(2) {
        draw_line(w[0].x, w[0].y, w[1].x, w[1].y, t, c);
    }
}

pub fn dashed(a: Vec2, b: Vec2, dash: f32, gap: f32, t: f32, c: Color) {
    let d = b - a;
    let len = d.length();
    if len < 0.01 {
        return;
    }
    let dir = d / len;
    let mut s = 0.0;
    while s < len {
        let e = (s + dash).min(len);
        draw_line(
            a.x + dir.x * s,
            a.y + dir.y * s,
            a.x + dir.x * e,
            a.y + dir.y * e,
            t,
            c,
        );
        s += dash + gap;
    }
}

pub fn circle_line(c: Vec2, r: f32, t: f32, col: Color) {
    let n = 36;
    for i in 0..n {
        let a0 = std::f32::consts::TAU * i as f32 / n as f32;
        let a1 = std::f32::consts::TAU * (i + 1) as f32 / n as f32;
        draw_line(
            c.x + r * a0.cos(),
            c.y + r * a0.sin(),
            c.x + r * a1.cos(),
            c.y + r * a1.sin(),
            t,
            col,
        );
    }
}

pub fn arc(c: Vec2, r: f32, a0: f32, a1: f32, t: f32, col: Color) {
    let n = ((a1 - a0).abs() / 0.12).ceil().max(4.0) as usize;
    for i in 0..n {
        let p0 = a0 + (a1 - a0) * i as f32 / n as f32;
        let p1 = a0 + (a1 - a0) * (i + 1) as f32 / n as f32;
        draw_line(
            c.x + r * p0.cos(),
            c.y + r * p0.sin(),
            c.x + r * p1.cos(),
            c.y + r * p1.sin(),
            t,
            col,
        );
    }
}

// --------------------------------------------------------------------- text

pub fn text(f: &Font, s: &str, x: f32, y: f32, size: f32, c: Color) {
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font: Some(f),
            font_size: size.max(9.6) as u16,
            font_scale: 1.0,
            color: c,
            ..Default::default()
        },
    );
}

pub fn text_w(f: &Font, s: &str, size: f32) -> f32 {
    measure_text(s, Some(f), size.max(9.6) as u16, 1.0).width
}

pub fn text_center(f: &Font, s: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = text_w(f, s, size);
    text(f, s, cx - w / 2.0, y, size, c);
}

pub fn text_right(f: &Font, s: &str, rx: f32, y: f32, size: f32, c: Color) {
    let w = text_w(f, s, size);
    text(f, s, rx - w, y, size, c);
}

pub fn wrap(f: &Font, s: &str, size: f32, max_w: f32) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let trial = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", cur, word)
        };
        if text_w(f, &trial, size) > max_w && !cur.is_empty() {
            out.push(cur.clone());
            cur = word.to_string();
        } else {
            cur = trial;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// -------------------------------------------------------------------- icons

fn rot(p: Vec2, a: f32) -> Vec2 {
    vec2(p.x * a.cos() - p.y * a.sin(), p.x * a.sin() + p.y * a.cos())
}

pub fn icon_back(c: Vec2, s: f32, t: f32, col: Color) {
    draw_line(c.x + s * 0.28, c.y - s * 0.45, c.x - s * 0.22, c.y, t, col);
    draw_line(c.x - s * 0.22, c.y, c.x + s * 0.28, c.y + s * 0.45, t, col);
}

pub fn icon_close(c: Vec2, s: f32, t: f32, col: Color) {
    draw_line(c.x - s * 0.35, c.y - s * 0.35, c.x + s * 0.35, c.y + s * 0.35, t, col);
    draw_line(c.x + s * 0.35, c.y - s * 0.35, c.x - s * 0.35, c.y + s * 0.35, t, col);
}

pub fn icon_check(c: Vec2, s: f32, t: f32, col: Color) {
    draw_line(c.x - s * 0.38, c.y, c.x - s * 0.08, c.y + s * 0.32, t, col);
    draw_line(c.x - s * 0.08, c.y + s * 0.32, c.x + s * 0.4, c.y - s * 0.34, t, col);
}

pub fn icon_play(c: Vec2, s: f32, col: Color) {
    draw_triangle(
        vec2(c.x - s * 0.28, c.y - s * 0.42),
        vec2(c.x - s * 0.28, c.y + s * 0.42),
        vec2(c.x + s * 0.45, c.y),
        col,
    );
}

pub fn icon_gear(c: Vec2, s: f32, t: f32, col: Color) {
    circle_line(c, s * 0.30, t, col);
    draw_circle(c.x, c.y, s * 0.10, col);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let r0 = s * 0.34;
        let r1 = s * 0.56;
        draw_line(
            c.x + a.cos() * r0,
            c.y + a.sin() * r0,
            c.x + a.cos() * r1,
            c.y + a.sin() * r1,
            t * 1.7,
            col,
        );
    }
}

pub fn icon_home(c: Vec2, s: f32, t: f32, col: Color) {
    let p = vec2(c.x, c.y - s * 0.42);
    let l = vec2(c.x - s * 0.5, c.y + s * 0.02);
    let r = vec2(c.x + s * 0.5, c.y + s * 0.02);
    draw_line(p.x, p.y, l.x, l.y, t, col);
    draw_line(p.x, p.y, r.x, r.y, t, col);
    draw_line(l.x, l.y, l.x, c.y + s * 0.42, t, col);
    draw_line(r.x, r.y, r.x, c.y + s * 0.42, t, col);
    draw_line(l.x, c.y + s * 0.42, r.x, c.y + s * 0.42, t, col);
    draw_line(c.x - s * 0.13, c.y + s * 0.42, c.x - s * 0.13, c.y + s * 0.08, t, col);
    draw_line(c.x + s * 0.13, c.y + s * 0.42, c.x + s * 0.13, c.y + s * 0.08, t, col);
    draw_line(c.x - s * 0.13, c.y + s * 0.08, c.x + s * 0.13, c.y + s * 0.08, t, col);
}

pub fn icon_gallery(c: Vec2, s: f32, t: f32, col: Color) {
    rrect_line(c.x - s * 0.52, c.y - s * 0.42, s * 0.86, s * 0.72, s * 0.16, t, col);
    draw_line(c.x - s * 0.32, c.y + s * 0.14, c.x - s * 0.06, c.y - s * 0.12, t, col);
    draw_line(c.x - s * 0.06, c.y - s * 0.12, c.x + s * 0.3, c.y + s * 0.24, t, col);
    draw_circle(c.x + s * 0.12, c.y - s * 0.2, s * 0.08, col);
}

pub fn icon_pencil(c: Vec2, s: f32, t: f32, col: Color) {
    let a = rot(vec2(-0.16, -0.5) * s, 0.0) + c;
    let b = rot(vec2(0.16, -0.5) * s, 0.0) + c;
    let c1 = c + vec2(0.34, 0.36) * s;
    let c2 = c + vec2(-0.02, 0.3) * s;
    draw_line(a.x, a.y, b.x, b.y, t, col);
    draw_line(a.x, a.y, c2.x, c2.y, t, col);
    draw_line(b.x, b.y, c1.x, c1.y, t, col);
    draw_line(c2.x, c2.y, c1.x, c1.y, t, col);
    draw_line(a.x + (c2.x - a.x) * 0.35, a.y + (c2.y - a.y) * 0.35, b.x + (c1.x - b.x) * 0.35, b.y + (c1.y - b.y) * 0.35, t, col);
    draw_triangle(
        vec2(c.x - 0.02 * s, c.y + 0.3 * s),
        vec2(c.x + 0.34 * s, c.y + 0.36 * s),
        vec2(c.x + 0.24 * s, c.y + 0.52 * s),
        col,
    );
}

pub fn icon_marker(c: Vec2, s: f32, t: f32, col: Color) {
    rrect(c.x - s * 0.16, c.y - s * 0.5, s * 0.32, s * 0.62, s * 0.08, col);
    draw_triangle(
        vec2(c.x - s * 0.16, c.y + s * 0.12),
        vec2(c.x + s * 0.16, c.y + s * 0.12),
        vec2(c.x + s * 0.06, c.y + s * 0.5),
        col,
    );
    draw_line(c.x - s * 0.16, c.y - s * 0.06, c.x + s * 0.16, c.y - s * 0.06, t, fade(SNOW, 0.9));
}

pub fn icon_brush(c: Vec2, s: f32, t: f32, col: Color) {
    draw_line(c.x + s * 0.3, c.y - s * 0.46, c.x + s * 0.02, c.y - s * 0.02, t, col);
    let tip = c + vec2(-0.12, 0.1) * s;
    draw_triangle(
        c + vec2(0.12, -0.12) * s,
        tip,
        c + vec2(-0.34, 0.48) * s,
        col,
    );
    draw_circle(c.x - 0.34 * s, c.y + 0.48 * s, t * 0.9, col);
}

pub fn icon_eraser(c: Vec2, s: f32, t: f32, col: Color) {
    let a = c + vec2(-0.44, 0.12) * s;
    let b = c + vec2(-0.1, -0.34) * s;
    let d = c + vec2(0.34, -0.02) * s;
    let e = c + vec2(0.0, 0.44) * s;
    draw_line(a.x, a.y, b.x, b.y, t, col);
    draw_line(b.x, b.y, d.x, d.y, t, col);
    draw_line(d.x, d.y, e.x, e.y, t, col);
    draw_line(e.x, e.y, a.x, a.y, t, col);
    draw_line(c.x - 0.26 * s, c.y + 0.28 * s, c.x + 0.16 * s, c.y - 0.16 * s, t, col);
}

pub fn icon_camera(c: Vec2, s: f32, t: f32, col: Color) {
    rrect_line(c.x - s * 0.46, c.y - s * 0.28, s * 0.92, s * 0.62, s * 0.14, t, col);
    circle_line(c, t + 0.5, t, col);
    draw_line(c.x - s * 0.2, c.y - s * 0.28, c.x - s * 0.1, c.y - s * 0.44, t, col);
    draw_line(c.x - s * 0.1, c.y - s * 0.44, c.x + s * 0.14, c.y - s * 0.44, t, col);
    draw_line(c.x + s * 0.14, c.y - s * 0.44, c.x + s * 0.24, c.y - s * 0.28, t, col);
}

pub fn icon_photo(c: Vec2, s: f32, t: f32, col: Color) {
    rrect_line(c.x - s * 0.46, c.y - s * 0.4, s * 0.92, s * 0.8, s * 0.12, t, col);
    draw_line(c.x - s * 0.32, c.y + s * 0.24, c.x - s * 0.04, c.y - s * 0.1, t, col);
    draw_line(c.x - s * 0.04, c.y - s * 0.1, c.x + s * 0.22, c.y + s * 0.24, t, col);
    draw_circle(c.x + s * 0.18, c.y - s * 0.16, s * 0.08, col);
}

pub fn icon_search(c: Vec2, s: f32, t: f32, col: Color) {
    circle_line(c + vec2(-0.06, -0.06) * s, s * 0.3, t, col);
    draw_line(c.x + 0.16 * s, c.y + 0.16 * s, c.x + 0.44 * s, c.y + 0.44 * s, t, col);
}

pub fn icon_target(c: Vec2, s: f32, t: f32, col: Color) {
    circle_line(c, s * 0.42, t, col);
    circle_line(c, s * 0.18, t, col);
    draw_line(c.x - s * 0.6, c.y, c.x - s * 0.44, c.y, t, col);
    draw_line(c.x + s * 0.44, c.y, c.x + s * 0.6, c.y, t, col);
    draw_line(c.x, c.y - s * 0.6, c.x, c.y - s * 0.44, t, col);
    draw_line(c.x, c.y + s * 0.44, c.x, c.y + s * 0.6, t, col);
}

pub fn icon_flash(c: Vec2, s: f32, col: Color) {
    draw_triangle(
        c + vec2(0.1, -0.52) * s,
        c + vec2(0.12, 0.04) * s,
        c + vec2(-0.18, 0.04) * s,
        col,
    );
    draw_triangle(
        c + vec2(-0.12, -0.04) * s,
        c + vec2(-0.12, 0.52) * s,
        c + vec2(0.18, -0.04) * s,
        col,
    );
    draw_triangle(
        c + vec2(-0.12, -0.04) * s,
        c + vec2(0.18, -0.04) * s,
        c + vec2(0.12, 0.04) * s,
        col,
    );
}

pub fn icon_lock(c: Vec2, s: f32, t: f32, col: Color, open: bool) {
    rrect_line(c.x - s * 0.36, c.y - s * 0.02, s * 0.72, s * 0.5, s * 0.1, t, col);
    let off = if open { s * 0.18 } else { 0.0 };
    arc(c + vec2(off, -s * 0.02), s * 0.22, std::f32::consts::PI, std::f32::consts::TAU, t, col);
}

pub fn icon_undo(c: Vec2, s: f32, t: f32, col: Color, flip: bool) {
    let sx = if flip { -1.0 } else { 1.0 };
    arc(c + vec2(0.0, 0.05) * s, s * 0.34, std::f32::consts::PI, std::f32::consts::TAU * 0.98, t, col);
    let tip = c + vec2(-0.34 * s * sx, 0.05 * s);
    draw_line(tip.x, tip.y, tip.x + 0.14 * s * sx, tip.y - 0.16 * s, t, col);
    draw_line(tip.x, tip.y, tip.x + 0.16 * s * sx, tip.y + 0.16 * s, t, col);
}

pub fn icon_trash(c: Vec2, s: f32, t: f32, col: Color) {
    draw_line(c.x - s * 0.4, c.y - s * 0.32, c.x + s * 0.4, c.y - s * 0.32, t, col);
    draw_line(c.x - s * 0.14, c.y - s * 0.44, c.x + s * 0.14, c.y - s * 0.44, t, col);
    draw_line(c.x - s * 0.3, c.y - s * 0.32, c.x - s * 0.24, c.y + s * 0.44, t, col);
    draw_line(c.x + s * 0.3, c.y - s * 0.32, c.x + s * 0.24, c.y + s * 0.44, t, col);
    draw_line(c.x - s * 0.24, c.y + s * 0.44, c.x + s * 0.24, c.y + s * 0.44, t, col);
    draw_line(c.x - s * 0.08, c.y - s * 0.2, c.x - s * 0.08, c.y + 0.3 * s, t, col);
    draw_line(c.x + s * 0.08, c.y - s * 0.2, c.x + s * 0.08, c.y + 0.3 * s, t, col);
}

pub fn icon_grid(c: Vec2, s: f32, t: f32, col: Color) {
    rrect_line(c.x - s * 0.42, c.y - s * 0.42, s * 0.84, s * 0.84, s * 0.1, t, col);
    draw_line(c.x - s * 0.14, c.y - s * 0.42, c.x - s * 0.14, c.y + s * 0.42, t, fade(col, 0.7));
    draw_line(c.x + s * 0.14, c.y - s * 0.42, c.x + s * 0.14, c.y + s * 0.42, t, fade(col, 0.7));
    draw_line(c.x - s * 0.42, c.y - s * 0.14, c.x + s * 0.42, c.y - s * 0.14, t, fade(col, 0.7));
    draw_line(c.x - s * 0.42, c.y + s * 0.14, c.x + s * 0.42, c.y + s * 0.14, t, fade(col, 0.7));
}

pub fn icon_eye(c: Vec2, s: f32, t: f32, col: Color, open: bool) {
    if !open {
        draw_line(c.x - s * 0.5, c.y + s * 0.3, c.x + s * 0.5, c.y - s * 0.18, t, col);
        return;
    }
    let upper = [
        c + vec2(-0.5, 0.02) * s,
        c + vec2(-0.22, -0.2) * s,
        c + vec2(0.22, -0.2) * s,
        c + vec2(0.5, 0.02) * s,
    ];
    let lower = [
        c + vec2(-0.5, 0.02) * s,
        c + vec2(-0.22, 0.24) * s,
        c + vec2(0.22, 0.24) * s,
        c + vec2(0.5, 0.02) * s,
    ];
    for i in 0..3 {
        draw_line(upper[i].x, upper[i].y, upper[i + 1].x, upper[i + 1].y, t, col);
        draw_line(lower[i].x, lower[i].y, lower[i + 1].x, lower[i + 1].y, t, col);
    }
    draw_circle(c.x, c.y + 0.03 * s, s * 0.12, col);
}

pub fn icon_sliders(c: Vec2, s: f32, t: f32, col: Color) {
    let rows = [(c.y - 0.28 * s, -0.16, 0.18), (c.y, 0.14, -0.2), (c.y + 0.28 * s, -0.02, 0.22)];
    for (y, knob_x, track_shift) in rows {
        let x0 = c.x - 0.42 * s + track_shift * 0.0;
        draw_line(x0, y, c.x + 0.42 * s, y, t, fade(col, 0.75));
        draw_circle(c.x + knob_x * s, y, 0.13 * s, col);
    }
}

pub fn icon_star(c: Vec2, s: f32, col: Color) {
    let mut pts = Vec::new();
    for i in 0..10 {
        let r = if i % 2 == 0 { s * 0.5 } else { s * 0.2 };
        let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
        pts.push(vec2(c.x + r * a.cos(), c.y + r * a.sin()));
    }
    for i in 0..10 {
        draw_triangle(c, pts[i], pts[(i + 1) % 10], col);
    }
}
