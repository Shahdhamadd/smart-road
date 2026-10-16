mod algorithm;
mod consts;
mod geometry;
mod path;
mod pixel;
mod render;
mod spawner;
mod stats;
mod vehicle;

use std::collections::HashSet;
use std::time::Instant;

use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;

use consts::*;
use geometry::Direction;
use path::PathTable;
use render::{Assets, PALETTE_LEN};
use stats::Stats;
use vehicle::Vehicle;

fn main() {
    let sdl_context = sdl2::init().expect("sdl2 init");
    let video = sdl_context.video().expect("sdl2 video");

    let window = video
        .window("Smart Road", WIN_W, WIN_H)
        .position_centered()
        .build()
        .expect("create window");

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().expect("create canvas");
    let texture_creator = canvas.texture_creator();
    let assets = Assets::new(&mut canvas, &texture_creator);

    let mut event_pump = sdl_context.event_pump().expect("event pump");

    if !show_menu(&mut canvas, &mut event_pump) {
        return;
    }

    let paths = PathTable::new();
    loop {
        let stats = run_simulation(&mut canvas, &assets, &paths, &mut event_pump);
        if !show_stats_screen(&mut canvas, &stats, &mut event_pump) {
            break;
        }
    }
}

fn run_simulation(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    assets: &Assets,
    paths: &PathTable,
    event_pump: &mut sdl2::EventPump,
) -> Stats {
    let mut vehicles: Vec<Vehicle> = Vec::new();
    let mut stats = Stats::new();
    let mut violations: HashSet<(u64, u64)> = HashSet::new();
    let mut rng: ThreadRng = rand::thread_rng();
    let mut next_id: u64 = 1;

    let mut sim_time: f64 = 0.0;
    let mut random_spawn = false;
    let mut random_spawn_timer = 0.0;

    let mut last_instant = Instant::now();
    let mut running = true;

    'running: loop {
        let now = Instant::now();
        let dt = (now - last_instant).as_secs_f64().min(0.05);
        last_instant = now;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    running = false;
                }
                Event::KeyDown { keycode: Some(kc), repeat: false, .. } => match kc {
                    Keycode::Escape => running = false,
                    Keycode::Up => try_spawn(Direction::South, &mut vehicles, paths, &mut next_id, &mut rng),
                    Keycode::Down => try_spawn(Direction::North, &mut vehicles, paths, &mut next_id, &mut rng),
                    Keycode::Right => try_spawn(Direction::West, &mut vehicles, paths, &mut next_id, &mut rng),
                    Keycode::Left => try_spawn(Direction::East, &mut vehicles, paths, &mut next_id, &mut rng),
                    Keycode::R => random_spawn = !random_spawn,
                    _ => {}
                },
                _ => {}
            }
        }

        if !running {
            break 'running;
        }

        sim_time += dt;

        if random_spawn {
            random_spawn_timer -= dt;
            if random_spawn_timer <= 0.0 {
                random_spawn_timer = RANDOM_SPAWN_INTERVAL;
                let entry = Direction::ALL[rng.gen_range(0..4)];
                try_spawn(entry, &mut vehicles, paths, &mut next_id, &mut rng);
            }
        }

        algorithm::update(&mut vehicles, dt, sim_time);

        for v in vehicles.iter() {
            stats.record_velocity_sample(v.velocity);
        }
        stats::update_close_calls(&vehicles, &mut stats, &mut violations);

        let mut i = 0;
        while i < vehicles.len() {
            if vehicles[i].is_done() {
                let v = vehicles.swap_remove(i);
                if let Some(detected_at) = v.detected_at {
                    stats.record_completion(sim_time - detected_at);
                }
            } else {
                i += 1;
            }
        }

        render::draw_scene(canvas);
        for v in vehicles.iter() {
            assets.draw_vehicle(canvas, v);
        }
        canvas.present();
    }

    stats
}

fn try_spawn(
    entry: Direction,
    vehicles: &mut Vec<Vehicle>,
    paths: &PathTable,
    next_id: &mut u64,
    rng: &mut ThreadRng,
) {
    if let Some(v) = spawner::spawn(entry, vehicles, paths, next_id, rng, PALETTE_LEN) {
        vehicles.push(v);
    }
}

const OVERLAY: Color = Color::RGBA(12, 14, 20, 200);
const CARD: Color = Color::RGB(30, 32, 48);
const CARD_BORDER: Color = Color::RGB(236, 236, 244);
const TILE: Color = Color::RGB(46, 50, 72);
const TILE_BORDER: Color = Color::RGB(84, 90, 122);
const DIVIDER: Color = Color::RGB(84, 90, 122);
const ACCENT: Color = Color::RGB(255, 205, 50);
const TEXT: Color = Color::RGB(255, 255, 255);
const SOFT: Color = Color::RGB(206, 212, 230);
const MUTED: Color = Color::RGB(142, 150, 180);
const BLUE: Color = Color::RGB(64, 160, 255);
const PURPLE: Color = Color::RGB(184, 110, 240);
const GREEN: Color = Color::RGB(56, 200, 96);
const RED: Color = Color::RGB(240, 80, 70);

type Screen = sdl2::render::Canvas<sdl2::video::Window>;

fn show_menu(canvas: &mut Screen, event_pump: &mut sdl2::EventPump) -> bool {
    const CONTROLS: [(&str, &str); 6] = [
        ("Up", "Vehicle from south"),
        ("Down", "Vehicle from north"),
        ("Right", "Vehicle from west"),
        ("Left", "Vehicle from east"),
        ("R", "Random vehicles"),
        ("Esc", "End and show summary"),
    ];

    let card = Rect::new(110, 70, 680, 760);
    let button = Rect::new((WIN_W as i32 - 360) / 2, 690, 360, 64);
    let start = Instant::now();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return false,
                Event::KeyDown { keycode: Some(Keycode::Return), .. }
                | Event::KeyDown { keycode: Some(Keycode::Space), .. } => return true,
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    if button.contains_point((x, y)) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        let mouse = event_pump.mouse_state();
        let hovered = button.contains_point((mouse.x(), mouse.y()));

        draw_backdrop(canvas);
        pixel::panel(canvas, card, CARD, CARD_BORDER);

        pixel::draw_title(canvas, "Smart Road", 112, 8, ACCENT);
        pixel::draw_text_centered(canvas, "Autonomous intersection simulation", 196, 2, SOFT);
        pixel::dashed_line(canvas, card.x() + 40, card.right() - 40, 236, DIVIDER);
        pixel::draw_text(canvas, "Controls", card.x() + 40, 262, 2, MUTED);

        let mut y = 298;
        for (key, action) in CONTROLS {
            let cap = Rect::new(card.x() + 40, y, 116, 44);
            pixel::panel(canvas, cap, TILE, TILE_BORDER);
            pixel::draw_text_in(canvas, key, cap, 3, TEXT);
            pixel::draw_text(canvas, action, cap.right() + 30, y + (44 - pixel::text_height(3)) / 2, 3, SOFT);
            y += 62;
        }

        pixel::button(canvas, button, "Start", hovered);
        if start.elapsed().as_millis() / 500 % 2 == 0 {
            pixel::draw_text_centered(canvas, "Press Enter or Space to start", 782, 2, TEXT);
        }

        canvas.present();
    }
}

/// Shows the end-of-run summary. Returns true if the user chose to start again.
fn show_stats_screen(canvas: &mut Screen, stats: &Stats, event_pump: &mut sdl2::EventPump) -> bool {
    let rows = stats.rows();
    let (hero, tiles) = rows.split_first().expect("stats rows");
    let close_calls_color = if stats.close_calls == 0 { GREEN } else { RED };
    let accents = [BLUE, BLUE, BLUE, PURPLE, PURPLE, close_calls_color];

    let card = Rect::new(110, 50, 680, 800);
    let hero_tile = Rect::new(150, 166, 600, 104);
    let button = Rect::new((WIN_W as i32 - 360) / 2, 682, 360, 64);
    const TILE_W: u32 = 290;
    const TILE_H: u32 = 104;
    const GAP: i32 = 20;

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return false,
                Event::KeyDown { keycode: Some(kc), repeat: false, .. } => match kc {
                    Keycode::Escape => return false,
                    Keycode::Return | Keycode::Space | Keycode::R => return true,
                    _ => {}
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    if button.contains_point((x, y)) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        let mouse = event_pump.mouse_state();
        let hovered = button.contains_point((mouse.x(), mouse.y()));

        draw_backdrop(canvas);
        pixel::panel(canvas, card, CARD, CARD_BORDER);

        pixel::draw_title(canvas, "Simulation Summary", 86, 5, ACCENT);
        pixel::dashed_line(canvas, card.x() + 40, card.right() - 40, 140, DIVIDER);

        pixel::panel(canvas, hero_tile, TILE, TILE_BORDER);
        let label_y = hero_tile.y() + (hero_tile.height() as i32 - pixel::text_height(3)) / 2;
        pixel::draw_text(canvas, hero.0, hero_tile.x() + 30, label_y, 3, SOFT);
        let value_x = hero_tile.right() - 30 - pixel::text_width(&hero.1, 7);
        let value_y = hero_tile.y() + (hero_tile.height() as i32 - pixel::text_height(7)) / 2;
        pixel::draw_text(canvas, &hero.1, value_x, value_y, 7, ACCENT);

        for (i, (label, value, unit)) in tiles.iter().enumerate() {
            let x = hero_tile.x() + (i as i32 % 2) * (TILE_W as i32 + GAP);
            let y = hero_tile.bottom() + GAP + (i as i32 / 2) * (TILE_H as i32 + GAP);
            pixel::panel(canvas, Rect::new(x, y, TILE_W, TILE_H), TILE, TILE_BORDER);
            canvas.set_draw_color(accents[i % accents.len()]);
            canvas.fill_rect(Rect::new(x + 16, y + 20, 8, TILE_H - 40)).ok();
            pixel::draw_text(canvas, label, x + 38, y + 22, 2, MUTED);
            let w = pixel::draw_text(canvas, value, x + 38, y + 52, 4, TEXT);
            if value != "N/A" {
                let unit_y = y + 52 + pixel::text_height(4) - pixel::text_height(2);
                pixel::draw_text(canvas, unit, x + 38 + w + 10, unit_y, 2, MUTED);
            }
        }

        pixel::button(canvas, button, "Start Again", hovered);
        pixel::draw_text_centered(canvas, "Enter: start again   Esc: quit", 790, 2, MUTED);

        canvas.present();
    }
}

/// The road scene dimmed behind a menu card.
fn draw_backdrop(canvas: &mut Screen) {
    render::draw_scene(canvas);
    canvas.set_blend_mode(BlendMode::Blend);
    canvas.set_draw_color(OVERLAY);
    canvas.fill_rect(None).ok();
    canvas.set_blend_mode(BlendMode::None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// Runs the controller headless under heavy random traffic and returns its stats.
    fn simulate(seed: u64, seconds: f64, spawn_interval: f64) -> Stats {
        let paths = PathTable::new();
        let mut rng = StdRng::seed_from_u64(seed);
        let mut vehicles: Vec<Vehicle> = Vec::new();
        let mut stats = Stats::new();
        let mut active = HashSet::new();
        let mut next_id = 1;
        let dt = 1.0 / 60.0;
        let mut sim_time = 0.0;
        let mut timer = 0.0;
        while sim_time < seconds {
            sim_time += dt;
            timer -= dt;
            if timer <= 0.0 {
                timer = spawn_interval;
                let entry = Direction::ALL[rng.gen_range(0..4)];
                if let Some(v) = spawner::spawn(entry, &vehicles, &paths, &mut next_id, &mut rng, PALETTE_LEN) {
                    vehicles.push(v);
                }
            }
            algorithm::update(&mut vehicles, dt, sim_time);
            stats::update_close_calls(&vehicles, &mut stats, &mut active);
            vehicles.retain(|v| {
                if v.is_done() {
                    stats.record_completion(sim_time - v.detected_at.expect("every path crosses the request zone"));
                }
                !v.is_done()
            });
        }
        stats
    }

    #[test]
    fn heavy_random_traffic_has_no_close_calls() {
        for seed in 0..8 {
            let stats = simulate(seed, 300.0, 0.25);
            assert!(stats.vehicles_passed > 300, "seed {seed}: only {} passed", stats.vehicles_passed);
            assert_eq!(stats.close_calls, 0, "seed {seed}");
        }
    }

    #[test]
    fn grid_cell_stays_in_range_on_box_edges() {
        assert_eq!(path::grid_cell(BOX_MAX, BOX_MAX), None);
        assert_eq!(path::grid_cell(BOX_MIN, BOX_MIN), Some((0, 0)));
        assert_eq!(path::grid_cell(BOX_MAX - 1e-9, BOX_MIN), Some((11, 0)));
    }
}
