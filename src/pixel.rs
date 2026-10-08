use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::WIN_W;

/// Size of one "pixel" block used for borders, corners and shadows.
pub const PX: i32 = 4;

const GLYPH_W: i32 = 5;
const GLYPH_H: i32 = 7;

const SHADOW: Color = Color::RGB(8, 9, 13);

/// 5x7 bitmap font, one row per byte with the leftmost pixel in bit 4.
fn glyph(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '/' => [0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000],
        _ => [0; 7],
    }
}

pub fn text_width(text: &str, scale: i32) -> i32 {
    let n = text.chars().count() as i32;
    if n == 0 {
        0
    } else {
        (n * (GLYPH_W + 1) - 1) * scale
    }
}

pub fn text_height(scale: i32) -> i32 {
    GLYPH_H * scale
}

/// Draws text with its top-left corner at (x, y) and returns the drawn width.
pub fn draw_text(canvas: &mut Canvas<Window>, text: &str, x: i32, y: i32, scale: i32, color: Color) -> i32 {
    let mut rects = Vec::new();
    for (i, c) in text.chars().enumerate() {
        let gx = x + i as i32 * (GLYPH_W + 1) * scale;
        for (row, bits) in glyph(c).iter().enumerate() {
            for col in 0..GLYPH_W {
                if bits & (1 << (GLYPH_W - 1 - col)) != 0 {
                    rects.push(Rect::new(gx + col * scale, y + row as i32 * scale, scale as u32, scale as u32));
                }
            }
        }
    }
    canvas.set_draw_color(color);
    canvas.fill_rects(&rects).ok();
    text_width(text, scale)
}

pub fn draw_text_centered(canvas: &mut Canvas<Window>, text: &str, y: i32, scale: i32, color: Color) {
    draw_text(canvas, text, (WIN_W as i32 - text_width(text, scale)) / 2, y, scale, color);
}

/// Centered text with a hard drop shadow, for titles.
pub fn draw_title(canvas: &mut Canvas<Window>, text: &str, y: i32, scale: i32, color: Color) {
    let x = (WIN_W as i32 - text_width(text, scale)) / 2;
    draw_text(canvas, text, x + scale, y + scale, scale, SHADOW);
    draw_text(canvas, text, x, y, scale, color);
}

pub fn draw_text_in(canvas: &mut Canvas<Window>, text: &str, rect: Rect, scale: i32, color: Color) {
    let x = rect.x() + (rect.width() as i32 - text_width(text, scale)) / 2;
    let y = rect.y() + (rect.height() as i32 - text_height(scale)) / 2;
    draw_text(canvas, text, x, y, scale, color);
}

/// A rectangle with one PX block cut from each corner.
fn stepped_rect(canvas: &mut Canvas<Window>, rect: Rect, color: Color) {
    let (x, y, w, h) = (rect.x(), rect.y(), rect.width() as i32, rect.height() as i32);
    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(x + PX, y, (w - 2 * PX) as u32, h as u32)).ok();
    canvas.fill_rect(Rect::new(x, y + PX, w as u32, (h - 2 * PX) as u32)).ok();
}

fn inset(rect: Rect, by: i32) -> Rect {
    Rect::new(rect.x() + by, rect.y() + by, rect.width() - 2 * by as u32, rect.height() - 2 * by as u32)
}

/// A bordered box with stepped corners and a hard drop shadow.
pub fn panel(canvas: &mut Canvas<Window>, rect: Rect, fill: Color, border: Color) {
    let mut shadow = rect;
    shadow.offset(2 * PX, 2 * PX);
    stepped_rect(canvas, shadow, SHADOW);
    stepped_rect(canvas, rect, border);
    stepped_rect(canvas, inset(rect, PX), fill);
}

pub fn button(canvas: &mut Canvas<Window>, rect: Rect, label: &str, hovered: bool) {
    let (fill, light, dark) = if hovered {
        (Color::RGB(88, 232, 120), Color::RGB(190, 255, 200), Color::RGB(40, 160, 78))
    } else {
        (Color::RGB(56, 200, 96), Color::RGB(140, 240, 160), Color::RGB(28, 128, 62))
    };
    panel(canvas, rect, fill, Color::RGB(10, 30, 18));
    let face = inset(rect, 2 * PX);
    canvas.set_draw_color(light);
    canvas.fill_rect(Rect::new(face.x(), face.y(), face.width(), PX as u32)).ok();
    canvas.set_draw_color(dark);
    canvas.fill_rect(Rect::new(face.x(), face.bottom() - 2 * PX, face.width(), 2 * PX as u32)).ok();
    draw_text_in(canvas, label, rect, 3, Color::RGB(10, 30, 18));
}

pub fn dashed_line(canvas: &mut Canvas<Window>, x0: i32, x1: i32, y: i32, color: Color) {
    canvas.set_draw_color(color);
    let mut x = x0;
    while x < x1 {
        canvas.fill_rect(Rect::new(x, y, (2 * PX).min(x1 - x) as u32, PX as u32)).ok();
        x += 4 * PX;
    }
}
