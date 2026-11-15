// The sketch library. Every picture is defined as polylines in a unit square
// (0..1 across, 0..1 down) so it can be drawn at any size. Each line carries
// the step it belongs to, which drives the step-by-step mode.

use crate::ui::fade;
use macroquad::prelude::*;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Path {
    pub pts: Vec<Vec2>,
    pub step: u8,
    pub weight: f32,
}

#[derive(Clone)]
pub struct Art {
    pub name: &'static str,
    pub topic: &'static str,
    pub difficulty: &'static str,
    pub paths: Vec<Path>,
}

fn p(pts: Vec<Vec2>, step: u8) -> Path {
    Path { pts, step, weight: 1.0 }
}

fn pw(pts: Vec<Vec2>, step: u8, weight: f32) -> Path {
    Path { pts, step, weight }
}

fn ellipse(c: Vec2, rx: f32, ry: f32, a0: f32, a1: f32, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            vec2(c.x + rx * a.cos(), c.y + ry * a.sin())
        })
        .collect()
}

fn ellipse_rot(c: Vec2, rx: f32, ry: f32, rot: f32, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let x = rx * a.cos();
            let y = ry * a.sin();
            vec2(
                c.x + x * rot.cos() - y * rot.sin(),
                c.y + x * rot.sin() + y * rot.cos(),
            )
        })
        .collect()
}

fn bez(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
        })
        .collect()
}

fn seg(a: Vec2, b: Vec2) -> Vec<Vec2> {
    vec![a, b]
}

fn poly(mut pts: Vec<Vec2>) -> Vec<Vec2> {
    if let (Some(f), Some(l)) = (pts.first().copied(), pts.last().copied()) {
        if (f - l).length() > 0.001 {
            pts.push(f);
        }
    }
    pts
}

fn v(x: f32, y: f32) -> Vec2 {
    vec2(x, y)
}

pub fn samples() -> Vec<Art> {
    vec![
        cat(),
        flower(),
        eye(),
        butterfly(),
        fish(),
        house(),
        fox(),
        portrait(),
        cup(),
        lighthouse(),
    ]
}

fn cat() -> Art {
    let mut paths = Vec::new();
    // head
    paths.push(p(ellipse(v(0.50, 0.40), 0.205, 0.205, 0.0, std::f32::consts::TAU, 60), 1));
    // ears
    paths.push(p(poly(vec![v(0.35, 0.27), v(0.30, 0.12), v(0.45, 0.22)]), 1));
    paths.push(p(poly(vec![v(0.65, 0.27), v(0.70, 0.12), v(0.55, 0.22)]), 1));
    paths.push(p(seg(v(0.34, 0.22), v(0.38, 0.19)), 1));
    paths.push(p(seg(v(0.66, 0.22), v(0.62, 0.19)), 1));
    // eyes
    paths.push(p(ellipse(v(0.425, 0.385), 0.055, 0.062, 0.0, std::f32::consts::TAU, 32), 2));
    paths.push(p(ellipse(v(0.575, 0.385), 0.055, 0.062, 0.0, std::f32::consts::TAU, 32), 2));
    paths.push(p(ellipse(v(0.435, 0.39), 0.020, 0.026, 0.0, std::f32::consts::TAU, 20), 3));
    paths.push(p(ellipse(v(0.565, 0.39), 0.020, 0.026, 0.0, std::f32::consts::TAU, 20), 3));
    // nose + mouth
    paths.push(p(poly(vec![v(0.468, 0.485), v(0.532, 0.485), v(0.50, 0.515)]), 2));
    paths.push(p(bez(v(0.50, 0.515), v(0.475, 0.555), v(0.445, 0.555), v(0.425, 0.535), 12), 2));
    paths.push(p(bez(v(0.50, 0.515), v(0.525, 0.555), v(0.555, 0.555), v(0.575, 0.535), 12), 2));
    // whiskers
    for (y, dy) in [(0.475, -0.02), (0.50, 0.0), (0.525, 0.02)] {
        paths.push(p(seg(v(0.365, y), v(0.175, y + dy)), 3));
        paths.push(p(seg(v(0.635, y), v(0.825, y + dy)), 3));
    }
    // body
    paths.push(p(bez(v(0.325, 0.56), v(0.28, 0.68), v(0.29, 0.80), v(0.315, 0.855), 16), 2));
    paths.push(p(bez(v(0.675, 0.56), v(0.72, 0.68), v(0.71, 0.80), v(0.685, 0.855), 16), 2));
    paths.push(p(bez(v(0.315, 0.855), v(0.40, 0.888), v(0.60, 0.888), v(0.685, 0.855), 20), 2));
    // paws
    paths.push(p(ellipse(v(0.375, 0.862), 0.060, 0.028, 0.0, std::f32::consts::TAU, 24), 3));
    paths.push(p(ellipse(v(0.625, 0.862), 0.060, 0.028, 0.0, std::f32::consts::TAU, 24), 3));
    // tail
    paths.push(p(bez(v(0.685, 0.82), v(0.83, 0.83), v(0.90, 0.72), v(0.82, 0.605), 18), 3));
    Art { name: "Cat", topic: "Animals", difficulty: "Easy", paths }
}

fn flower() -> Art {
    let mut paths = Vec::new();
    let c = v(0.50, 0.34);
    for i in 0..5 {
        let a = -PI / 2.0 + i as f32 * std::f32::consts::TAU / 5.0;
        let pc = c + v(a.cos(), a.sin()) * 0.155;
        paths.push(p(ellipse_rot(pc, 0.115, 0.070, a, 34), 1));
    }
    paths.push(p(ellipse(c, 0.058, 0.058, 0.0, std::f32::consts::TAU, 26), 3));
    for i in 0..7 {
        let a = i as f32 * std::f32::consts::TAU / 7.0;
        paths.push(p(seg(c + v(a.cos(), a.sin()) * 0.012, c + v(a.cos(), a.sin()) * 0.042), 3));
    }
    // stem
    paths.push(p(bez(v(0.50, 0.40), v(0.47, 0.56), v(0.53, 0.72), v(0.50, 0.92), 22), 2));
    // leaves
    paths.push(p(bez(v(0.495, 0.64), v(0.38, 0.58), v(0.28, 0.66), v(0.335, 0.74), 18), 2));
    paths.push(p(bez(v(0.495, 0.64), v(0.40, 0.67), v(0.34, 0.70), v(0.335, 0.74), 12), 2));
    paths.push(p(bez(v(0.505, 0.78), v(0.62, 0.72), v(0.72, 0.80), v(0.665, 0.88), 18), 2));
    paths.push(p(bez(v(0.505, 0.78), v(0.60, 0.81), v(0.66, 0.84), v(0.665, 0.88), 12), 2));
    Art { name: "Flower", topic: "Flowers", difficulty: "Easy", paths }
}

fn eye() -> Art {
    let mut paths = Vec::new();
    paths.push(p(bez(v(0.13, 0.47), v(0.34, 0.28), v(0.66, 0.28), v(0.87, 0.47), 26), 1));
    paths.push(p(bez(v(0.13, 0.47), v(0.34, 0.60), v(0.66, 0.60), v(0.87, 0.47), 26), 1));
    paths.push(p(ellipse(v(0.50, 0.455), 0.135, 0.135, 0.0, std::f32::consts::TAU, 40), 2));
    paths.push(p(ellipse(v(0.50, 0.455), 0.055, 0.055, 0.0, std::f32::consts::TAU, 24), 2));
    paths.push(p(ellipse(v(0.462, 0.418), 0.022, 0.022, 0.0, std::f32::consts::TAU, 16), 2));
    for i in 0..14 {
        let a = i as f32 * std::f32::consts::TAU / 14.0;
        paths.push(p(seg(v(0.50 + a.cos() * 0.075, 0.455 + a.sin() * 0.075), v(0.50 + a.cos() * 0.128, 0.455 + a.sin() * 0.128)), 3, ));
    }
    // lashes
    for i in 0..6 {
        let t = 0.18 + i as f32 * 0.13;
        let base = bez(v(0.13, 0.47), v(0.34, 0.28), v(0.66, 0.28), v(0.87, 0.47), 26)
            [(t * 26.0) as usize];
        paths.push(p(seg(base, base + v(-0.02 + i as f32 * 0.008, -0.055)), 3));
    }
    paths.push(p(bez(v(0.10, 0.24), v(0.36, 0.13), v(0.72, 0.15), v(0.91, 0.26), 22), 3));
    paths.push(p(bez(v(0.17, 0.34), v(0.40, 0.25), v(0.72, 0.27), v(0.86, 0.35), 20), 3));
    Art { name: "Eye", topic: "Faces", difficulty: "Medium", paths }
}

fn butterfly() -> Art {
    let mut paths = Vec::new();
    paths.push(p(bez(v(0.49, 0.44), v(0.36, 0.18), v(0.10, 0.22), v(0.16, 0.46), 24), 1));
    paths.push(p(bez(v(0.16, 0.46), v(0.22, 0.55), v(0.38, 0.53), v(0.49, 0.50), 18), 1));
    paths.push(p(bez(v(0.51, 0.44), v(0.64, 0.18), v(0.90, 0.22), v(0.84, 0.46), 24), 1));
    paths.push(p(bez(v(0.84, 0.46), v(0.78, 0.55), v(0.62, 0.53), v(0.51, 0.50), 18), 1));
    paths.push(p(bez(v(0.49, 0.55), v(0.34, 0.58), v(0.20, 0.70), v(0.30, 0.80), 22), 1));
    paths.push(p(bez(v(0.30, 0.80), v(0.40, 0.78), v(0.46, 0.68), v(0.49, 0.62), 16), 1));
    paths.push(p(bez(v(0.51, 0.55), v(0.66, 0.58), v(0.80, 0.70), v(0.70, 0.80), 22), 1));
    paths.push(p(bez(v(0.70, 0.80), v(0.60, 0.78), v(0.54, 0.68), v(0.51, 0.62), 16), 1));
    paths.push(p(ellipse(v(0.50, 0.52), 0.024, 0.175, 0.0, std::f32::consts::TAU, 30), 2));
    paths.push(p(ellipse(v(0.50, 0.345), 0.034, 0.034, 0.0, std::f32::consts::TAU, 20), 2));
    paths.push(p(bez(v(0.48, 0.325), v(0.43, 0.24), v(0.38, 0.19), v(0.33, 0.17), 12), 2));
    paths.push(p(bez(v(0.52, 0.325), v(0.57, 0.24), v(0.62, 0.19), v(0.67, 0.17), 12), 2));
    paths.push(p(ellipse(v(0.30, 0.365), 0.045, 0.040, 0.0, std::f32::consts::TAU, 22), 3));
    paths.push(p(ellipse(v(0.70, 0.365), 0.045, 0.040, 0.0, std::f32::consts::TAU, 22), 3));
    paths.push(p(ellipse(v(0.34, 0.715), 0.032, 0.030, 0.0, std::f32::consts::TAU, 18), 3));
    paths.push(p(ellipse(v(0.66, 0.715), 0.032, 0.030, 0.0, std::f32::consts::TAU, 18), 3));
    Art { name: "Butterfly", topic: "Animals", difficulty: "Medium", paths }
}

fn fish() -> Art {
    let mut paths = Vec::new();
    paths.push(p(ellipse(v(0.50, 0.50), 0.255, 0.150, 0.0, std::f32::consts::TAU, 44), 1));
    paths.push(p(poly(vec![v(0.26, 0.50), v(0.07, 0.35), v(0.11, 0.50), v(0.07, 0.65)]), 1));
    paths.push(p(bez(v(0.42, 0.36), v(0.49, 0.25), v(0.60, 0.28), v(0.63, 0.36), 16), 2));
    paths.push(p(bez(v(0.44, 0.64), v(0.50, 0.72), v(0.58, 0.71), v(0.61, 0.64), 14), 2));
    paths.push(p(ellipse(v(0.655, 0.455), 0.027, 0.027, 0.0, std::f32::consts::TAU, 20), 2));
    paths.push(p(ellipse(v(0.662, 0.458), 0.012, 0.012, 0.0, std::f32::consts::TAU, 14), 3));
    paths.push(p(arc_pts(v(0.615, 0.50), 0.085, -PI * 0.55, PI * 0.55), 2));
    paths.push(p(bez(v(0.75, 0.45), v(0.78, 0.47), v(0.78, 0.52), v(0.75, 0.545), 10), 3));
    for (i, cx) in [0.40, 0.46, 0.52].iter().enumerate() {
        let cy = 0.50 + (i as f32 - 1.0) * 0.045;
        paths.push(p(arc_pts(v(*cx, cy), 0.048, -PI * 0.42, PI * 0.42), 3));
    }
    paths.push(p(ellipse(v(0.20, 0.30), 0.018, 0.018, 0.0, std::f32::consts::TAU, 12), 3));
    paths.push(p(ellipse(v(0.26, 0.23), 0.013, 0.013, 0.0, std::f32::consts::TAU, 12), 3));
    Art { name: "Fish", topic: "Animals", difficulty: "Easy", paths }
}

fn arc_pts(c: Vec2, r: f32, a0: f32, a1: f32) -> Vec<Vec2> {
    let n = 24;
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            v(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

fn house() -> Art {
    let mut paths = Vec::new();
    paths.push(p(poly(vec![v(0.18, 0.44), v(0.18, 0.84), v(0.80, 0.84), v(0.80, 0.44)]), 1));
    paths.push(p(poly(vec![v(0.11, 0.45), v(0.49, 0.16), v(0.87, 0.45)]), 1));
    paths.push(p(poly(vec![v(0.42, 0.84), v(0.42, 0.64), v(0.565, 0.64), v(0.565, 0.84)]), 2));
    paths.push(p(poly(vec![v(0.24, 0.52), v(0.37, 0.52), v(0.37, 0.65), v(0.24, 0.65)]), 2));
    paths.push(p(poly(vec![v(0.615, 0.52), v(0.745, 0.52), v(0.745, 0.65), v(0.615, 0.65)]), 2));
    paths.push(p(seg(v(0.305, 0.52), v(0.305, 0.65)), 3));
    paths.push(p(seg(v(0.24, 0.585), v(0.37, 0.585)), 3));
    paths.push(p(seg(v(0.68, 0.52), v(0.68, 0.65)), 3));
    paths.push(p(seg(v(0.615, 0.585), v(0.745, 0.585)), 3));
    paths.push(p(poly(vec![v(0.655, 0.37), v(0.655, 0.25), v(0.735, 0.25), v(0.735, 0.42)]), 2));
    // tree
    paths.push(p(seg(v(0.10, 0.84), v(0.10, 0.72)), 3));
    paths.push(p(ellipse(v(0.10, 0.62), 0.085, 0.090, 0.0, std::f32::consts::TAU, 28), 3));
    paths.push(p(ellipse(v(0.155, 0.66), 0.060, 0.060, 0.0, std::f32::consts::TAU, 22), 3));
    // smoke + ground
    paths.push(p(bez(v(0.695, 0.23), v(0.66, 0.18), v(0.73, 0.15), v(0.70, 0.10), 14), 3));
    paths.push(p(seg(v(0.04, 0.84), v(0.96, 0.84)), 3, ));
    Art { name: "House", topic: "Places", difficulty: "Easy", paths }
}

fn fox() -> Art {
    let mut paths = Vec::new();
    paths.push(p(poly(vec![v(0.50, 0.26), v(0.73, 0.40), v(0.685, 0.61), v(0.50, 0.71), v(0.315, 0.61), v(0.27, 0.40)]), 1));
    paths.push(p(poly(vec![v(0.30, 0.41), v(0.25, 0.15), v(0.46, 0.29)]), 1));
    paths.push(p(poly(vec![v(0.70, 0.41), v(0.75, 0.15), v(0.54, 0.29)]), 1));
    paths.push(p(seg(v(0.315, 0.30), v(0.355, 0.34)), 2));
    paths.push(p(seg(v(0.685, 0.30), v(0.645, 0.34)), 2));
    paths.push(p(ellipse(v(0.425, 0.455), 0.047, 0.036, 0.0, std::f32::consts::TAU, 22), 2));
    paths.push(p(ellipse(v(0.575, 0.455), 0.047, 0.036, 0.0, std::f32::consts::TAU, 22), 2));
    paths.push(p(ellipse(v(0.428, 0.458), 0.018, 0.024, 0.0, std::f32::consts::TAU, 16), 3));
    paths.push(p(ellipse(v(0.572, 0.458), 0.018, 0.024, 0.0, std::f32::consts::TAU, 16), 3));
    paths.push(p(poly(vec![v(0.455, 0.545), v(0.545, 0.545), v(0.50, 0.60)]), 2));
    paths.push(p(ellipse(v(0.50, 0.545), 0.032, 0.020, 0.0, std::f32::consts::TAU, 16), 2));
    paths.push(p(bez(v(0.32, 0.545), v(0.27, 0.55), v(0.255, 0.59), v(0.27, 0.62), 10), 3));
    paths.push(p(bez(v(0.68, 0.545), v(0.73, 0.55), v(0.745, 0.59), v(0.73, 0.62), 10), 3));
    paths.push(p(bez(v(0.345, 0.66), v(0.31, 0.75), v(0.33, 0.83), v(0.36, 0.88), 14), 2));
    paths.push(p(bez(v(0.655, 0.66), v(0.69, 0.75), v(0.67, 0.83), v(0.64, 0.88), 14), 2));
    paths.push(p(bez(v(0.36, 0.88), v(0.43, 0.91), v(0.57, 0.91), v(0.64, 0.88), 16), 2));
    Art { name: "Fox", topic: "Animals", difficulty: "Hard", paths }
}

fn portrait() -> Art {
    let mut paths = Vec::new();
    paths.push(p(ellipse(v(0.50, 0.40), 0.165, 0.205, 0.0, std::f32::consts::TAU, 54), 1));
    // hair
    paths.push(p(bez(v(0.335, 0.36), v(0.26, 0.20), v(0.42, 0.10), v(0.50, 0.11), 20), 1));
    paths.push(p(bez(v(0.50, 0.11), v(0.60, 0.10), v(0.75, 0.19), v(0.665, 0.37), 20), 1));
    paths.push(p(bez(v(0.335, 0.38), v(0.26, 0.52), v(0.24, 0.70), v(0.30, 0.86), 22), 1));
    paths.push(p(bez(v(0.665, 0.38), v(0.74, 0.52), v(0.76, 0.73), v(0.70, 0.86), 22), 1));
    paths.push(p(bez(v(0.40, 0.16), v(0.38, 0.30), v(0.36, 0.44), v(0.345, 0.58), 16), 3));
    paths.push(p(bez(v(0.60, 0.16), v(0.63, 0.31), v(0.645, 0.44), v(0.655, 0.58), 16), 3));
    // eyes
    paths.push(p(ellipse(v(0.425, 0.395), 0.048, 0.028, 0.0, std::f32::consts::TAU, 24), 2));
    paths.push(p(ellipse(v(0.575, 0.395), 0.048, 0.028, 0.0, std::f32::consts::TAU, 24), 2));
    paths.push(p(ellipse(v(0.425, 0.395), 0.018, 0.018, 0.0, std::f32::consts::TAU, 14), 2));
    paths.push(p(ellipse(v(0.575, 0.395), 0.018, 0.018, 0.0, std::f32::consts::TAU, 14), 2));
    paths.push(p(bez(v(0.365, 0.355), v(0.40, 0.335), v(0.45, 0.335), v(0.482, 0.352), 12), 2));
    paths.push(p(bez(v(0.518, 0.352), v(0.55, 0.335), v(0.60, 0.335), v(0.635, 0.355), 12), 2));
    // brows
    paths.push(p(bez(v(0.365, 0.325), v(0.40, 0.30), v(0.45, 0.30), v(0.485, 0.318), 12), 3));
    paths.push(p(bez(v(0.515, 0.318), v(0.55, 0.30), v(0.60, 0.30), v(0.635, 0.325), 12), 3));
    // nose + lips
    paths.push(p(bez(v(0.50, 0.41), v(0.487, 0.47), v(0.478, 0.495), v(0.492, 0.505), 12), 2));
    paths.push(p(bez(v(0.492, 0.505), v(0.50, 0.512), v(0.51, 0.512), v(0.518, 0.505), 8), 2));
    paths.push(p(bez(v(0.445, 0.565), v(0.475, 0.552), v(0.525, 0.552), v(0.555, 0.565), 14), 3));
    paths.push(p(bez(v(0.445, 0.565), v(0.475, 0.592), v(0.525, 0.592), v(0.555, 0.565), 14), 3));
    // neck + shoulders
    paths.push(p(seg(v(0.44, 0.60), v(0.435, 0.70)), 2));
    paths.push(p(seg(v(0.56, 0.60), v(0.565, 0.70)), 2));
    paths.push(p(bez(v(0.435, 0.70), v(0.33, 0.74), v(0.24, 0.80), v(0.20, 0.90), 18), 3));
    paths.push(p(bez(v(0.565, 0.70), v(0.67, 0.74), v(0.76, 0.80), v(0.80, 0.90), 18), 3));
    paths.push(p(ellipse(v(0.345, 0.525), 0.012, 0.012, 0.0, std::f32::consts::TAU, 10), 3));
    Art { name: "Portrait", topic: "Faces", difficulty: "Hard", paths }
}

fn cup() -> Art {
    let mut paths = Vec::new();
    paths.push(p(poly(vec![v(0.30, 0.38), v(0.355, 0.72), v(0.645, 0.72), v(0.70, 0.38)]), 1));
    paths.push(p(ellipse(v(0.50, 0.38), 0.20, 0.045, 0.0, std::f32::consts::TAU, 30), 1));
    paths.push(p(ellipse(v(0.50, 0.385), 0.165, 0.030, 0.0, std::f32::consts::TAU, 26), 2));
    paths.push(p(arc_pts(v(0.70, 0.50), 0.105, -PI * 0.48, PI * 0.48), 2));
    paths.push(p(ellipse(v(0.50, 0.755), 0.285, 0.040, 0.0, std::f32::consts::TAU, 34), 2));
    paths.push(p(bez(v(0.42, 0.30), v(0.38, 0.24), v(0.46, 0.20), v(0.42, 0.13), 14), 3));
    paths.push(p(bez(v(0.50, 0.29), v(0.46, 0.22), v(0.54, 0.19), v(0.50, 0.11), 14), 3));
    paths.push(p(bez(v(0.58, 0.30), v(0.54, 0.24), v(0.62, 0.20), v(0.58, 0.14), 14), 3));
    Art { name: "Cup", topic: "Objects", difficulty: "Easy", paths }
}

fn lighthouse() -> Art {
    let mut paths = Vec::new();
    paths.push(p(poly(vec![v(0.415, 0.31), v(0.345, 0.82), v(0.655, 0.82), v(0.585, 0.31)]), 1));
    paths.push(p(poly(vec![v(0.385, 0.20), v(0.615, 0.20), v(0.615, 0.31), v(0.385, 0.31)]), 1));
    paths.push(p(poly(vec![v(0.375, 0.20), v(0.50, 0.105), v(0.625, 0.20)]), 1));
    paths.push(p(seg(v(0.375, 0.47), v(0.625, 0.47)), 2));
    paths.push(p(seg(v(0.36, 0.60), v(0.64, 0.60)), 2));
    paths.push(p(seg(v(0.352, 0.71), v(0.648, 0.71)), 2));
    paths.push(p(seg(v(0.50, 0.20), v(0.50, 0.31)), 2));
    // light
    paths.push(p(seg(v(0.615, 0.24), v(0.80, 0.16)), 3));
    paths.push(p(seg(v(0.615, 0.26), v(0.82, 0.26)), 3));
    paths.push(p(seg(v(0.385, 0.24), v(0.20, 0.16)), 3));
    paths.push(p(seg(v(0.385, 0.26), v(0.18, 0.26)), 3));
    // rocks + sea
    paths.push(p(poly(vec![v(0.20, 0.82), v(0.27, 0.77), v(0.34, 0.82)]), 3));
    paths.push(p(poly(vec![v(0.66, 0.82), v(0.74, 0.77), v(0.81, 0.82)]), 3));
    paths.push(p(bez(v(0.08, 0.88), v(0.30, 0.84), v(0.70, 0.92), v(0.94, 0.87), 20), 3));
    paths.push(p(bez(v(0.10, 0.94), v(0.34, 0.90), v(0.66, 0.97), v(0.92, 0.93), 20), 3));
    Art { name: "Lighthouse", topic: "Places", difficulty: "Medium", paths }
}

// Draw the art into a square. Lines above `max_step` are skipped. Step 4 is the
// finished, coloured stage the original ships as assets/step_by_step/step_4.webp.
pub fn draw_art(
    art: &Art,
    center: Vec2,
    side: f32,
    rotation: f32,
    alpha: f32,
    color: Color,
    line_scale: f32,
    max_step: u8,
) {
    if max_step >= 4 {
        draw_paths(
            art,
            center,
            side,
            rotation,
            alpha * 0.22,
            Color::new(0.44, 0.60, 0.82, 1.0),
            7.5 * line_scale,
            3,
        );
    }
    draw_paths(art, center, side, rotation, alpha, color, 1.6 * line_scale, max_step);
}

fn draw_paths(
    art: &Art,
    center: Vec2,
    side: f32,
    rotation: f32,
    alpha: f32,
    color: Color,
    width: f32,
    max_step: u8,
) {
    let s = side;
    for path in &art.paths {
        if path.step > max_step {
            continue;
        }
        let mut prev: Option<Vec2> = None;
        for pt in &path.pts {
            let local = vec2((pt.x - 0.5) * s, (pt.y - 0.5) * s);
            let r = vec2(
                local.x * rotation.cos() - local.y * rotation.sin(),
                local.x * rotation.sin() + local.y * rotation.cos(),
            );
            let p = center + r;
            if let Some(q) = prev {
                draw_line(q.x, q.y, p.x, p.y, width, fade(color, alpha));
            }
            prev = Some(p);
        }
    }
}
