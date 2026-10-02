use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::consts::*;
use crate::vehicle::Vehicle;

pub const PALETTE: [Color; 8] = [
    Color::RGB(231, 76, 60),
    Color::RGB(52, 152, 219),
    Color::RGB(46, 204, 113),
    Color::RGB(241, 196, 15),
    Color::RGB(155, 89, 182),
    Color::RGB(230, 126, 34),
    Color::RGB(26, 188, 156),
    Color::RGB(236, 240, 241),
];
pub const PALETTE_LEN: usize = PALETTE.len();

pub struct Assets<'a> {
    car_textures: Vec<Texture<'a>>,
}

impl<'a> Assets<'a> {
    pub fn new(canvas: &mut Canvas<Window>, texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        let mut car_textures = Vec::with_capacity(PALETTE_LEN);
        for &color in PALETTE.iter() {
            let mut tex = texture_creator
                .create_texture_target(PixelFormatEnum::RGBA8888, CAR_WID as u32, CAR_LEN as u32)
                .expect("create car texture");
            tex.set_blend_mode(BlendMode::Blend);
            canvas
                .with_texture_canvas(&mut tex, |tc| {
                    tc.set_draw_color(Color::RGBA(0, 0, 0, 0));
                    tc.clear();
                    let w = CAR_WID as u32;
                    let h = CAR_LEN as u32;
                    tc.set_draw_color(color);
                    tc.fill_rect(Rect::new(0, 0, w, h)).ok();
                    // windshield near the front (texture faces "up")
                    tc.set_draw_color(Color::RGB(35, 35, 45));
                    let wind_w = (w as f64 * 0.7) as u32;
                    let wind_h = (h as f64 * 0.28) as u32;
                    let wind_x = ((w.saturating_sub(wind_w)) / 2) as i32;
                    let wind_y = (h as f64 * 0.12) as i32;
                    tc.fill_rect(Rect::new(wind_x, wind_y, wind_w, wind_h)).ok();
                })
                .expect("render car texture");
            car_textures.push(tex);
        }
        Assets { car_textures }
    }

    pub fn draw_vehicle(&self, canvas: &mut Canvas<Window>, v: &Vehicle) {
        let tex = &self.car_textures[v.color_idx % self.car_textures.len()];
        let dst = Rect::new(
            (v.x - CAR_WID / 2.0).round() as i32,
            (v.y - CAR_LEN / 2.0).round() as i32,
            CAR_WID as u32,
            CAR_LEN as u32,
        );
        canvas.copy_ex(tex, None, Some(dst), v.angle, None, false, false).ok();
    }
}

pub fn draw_scene(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(70, 148, 74)); // grass
    canvas.clear();

    let road_color = Color::RGB(58, 58, 63);
    canvas.set_draw_color(road_color);
    canvas
        .fill_rect(Rect::new(BOX_MIN as i32, 0, (2.0 * APPROACH) as u32, WIN_H))
        .ok();
    canvas
        .fill_rect(Rect::new(0, BOX_MIN as i32, WIN_W, (2.0 * APPROACH) as u32))
        .ok();

    canvas.set_draw_color(Color::RGB(212, 202, 90));
    for i in 1..3 {
        let x = BOX_MIN + i as f64 * LANE_W;
        draw_dashed_v(canvas, x, 0.0, BOX_MIN);
        draw_dashed_v(canvas, x, BOX_MAX, WIN_H as f64);
        let y = BOX_MIN + i as f64 * LANE_W;
        draw_dashed_h(canvas, y, 0.0, BOX_MIN);
        draw_dashed_h(canvas, y, BOX_MAX, WIN_W as f64);
    }

    canvas.set_draw_color(Color::RGB(225, 225, 225));
    for &x in &[BOX_MIN, BOX_MAX] {
        canvas.draw_line((x as i32, 0), (x as i32, BOX_MIN as i32)).ok();
        canvas.draw_line((x as i32, BOX_MAX as i32), (x as i32, WIN_H as i32)).ok();
    }
    for &y in &[BOX_MIN, BOX_MAX] {
        canvas.draw_line((0, y as i32), (BOX_MIN as i32, y as i32)).ok();
        canvas.draw_line((BOX_MAX as i32, y as i32), (WIN_W as i32, y as i32)).ok();
    }
}

fn draw_dashed_v(canvas: &mut Canvas<Window>, x: f64, y0: f64, y1: f64) {
    let (dash, gap) = (14.0, 10.0);
    let mut y = y0;
    while y < y1 {
        let y_end = (y + dash).min(y1);
        canvas.draw_line((x as i32, y as i32), (x as i32, y_end as i32)).ok();
        y += dash + gap;
    }
}

fn draw_dashed_h(canvas: &mut Canvas<Window>, y: f64, x0: f64, x1: f64) {
    let (dash, gap) = (14.0, 10.0);
    let mut x = x0;
    while x < x1 {
        let x_end = (x + dash).min(x1);
        canvas.draw_line((x as i32, y as i32), (x_end as i32, y as i32)).ok();
        x += dash + gap;
    }
}
