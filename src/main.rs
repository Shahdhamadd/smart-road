mod algorithm;
mod consts;
mod geometry;
mod path;
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
use sdl2::pixels::Color;

use consts::*;
use geometry::Direction;
use path::PathTable;
use render::{Assets, PALETTE_LEN};
use stats::Stats;
use vehicle::Vehicle;

const FONT_CANDIDATES: [&str; 3] = [
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/System/Library/Fonts/SFNSMono.ttf",
];

fn find_font() -> Option<&'static str> {
    FONT_CANDIDATES.iter().copied().find(|p| std::path::Path::new(p).exists())
}

fn main() {
    let sdl_context = sdl2::init().expect("sdl2 init");
    let video = sdl_context.video().expect("sdl2 video");
    let ttf_context = sdl2::ttf::init().expect("sdl2 ttf init");

    let window = video
        .window("Smart Road", WIN_W, WIN_H)
        .position_centered()
        .build()
        .expect("create window");

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().expect("create canvas");
    let texture_creator = canvas.texture_creator();
    let assets = Assets::new(&mut canvas, &texture_creator);

    let mut event_pump = sdl_context.event_pump().expect("event pump");

    let paths = PathTable::new();
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
                    Keycode::Up => try_spawn(Direction::South, &mut vehicles, &paths, sim_time, &mut next_id, &mut rng),
                    Keycode::Down => try_spawn(Direction::North, &mut vehicles, &paths, sim_time, &mut next_id, &mut rng),
                    Keycode::Right => try_spawn(Direction::West, &mut vehicles, &paths, sim_time, &mut next_id, &mut rng),
                    Keycode::Left => try_spawn(Direction::East, &mut vehicles, &paths, sim_time, &mut next_id, &mut rng),
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
                try_spawn(entry, &mut vehicles, &paths, sim_time, &mut next_id, &mut rng);
            }
        }

        algorithm::update(&mut vehicles, dt);

        for v in vehicles.iter() {
            stats.record_velocity_sample(v.velocity);
        }
        stats::update_close_calls(&vehicles, &mut stats, &mut violations);

        let mut i = 0;
        while i < vehicles.len() {
            if vehicles[i].is_done() {
                let v = vehicles.swap_remove(i);
                stats.record_completion(sim_time - v.spawn_time);
            } else {
                i += 1;
            }
        }

        render::draw_scene(&mut canvas);
        for v in vehicles.iter() {
            assets.draw_vehicle(&mut canvas, v);
        }
        canvas.present();
    }

    show_stats_screen(&mut canvas, &texture_creator, &ttf_context, &stats, &mut event_pump);
}

fn try_spawn(
    entry: Direction,
    vehicles: &mut Vec<Vehicle>,
    paths: &PathTable,
    sim_time: f64,
    next_id: &mut u64,
    rng: &mut ThreadRng,
) {
    if let Some(v) = spawner::spawn(entry, vehicles, paths, sim_time, next_id, rng, PALETTE_LEN) {
        vehicles.push(v);
    }
}

fn show_stats_screen(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ttf_context: &sdl2::ttf::Sdl2TtfContext,
    stats: &Stats,
    event_pump: &mut sdl2::EventPump,
) {
    let font = find_font().map(|p| ttf_context.load_font(p, 28).expect("load font"));
    let title_font = find_font().map(|p| ttf_context.load_font(p, 40).expect("load font"));

    'stats: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'stats,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                | Event::KeyDown { keycode: Some(Keycode::Return), .. } => break 'stats,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(24, 26, 32));
        canvas.clear();

        let mut y = 120;
        if let Some(tf) = &title_font {
            draw_text(canvas, texture_creator, tf, "Simulation Statistics", 80, y, Color::RGB(255, 255, 255));
            y += 70;
        }
        if let Some(f) = &font {
            for line in stats.lines() {
                draw_text(canvas, texture_creator, f, &line, 90, y, Color::RGB(220, 220, 220));
                y += 44;
            }
            y += 30;
            draw_text(
                canvas,
                texture_creator,
                f,
                "Press ESC or Enter to close",
                90,
                y,
                Color::RGB(150, 150, 150),
            );
        }

        canvas.present();
    }
}

fn draw_text(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    font: &sdl2::ttf::Font,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) {
    let surface = font.render(text).blended(color).expect("render text");
    let texture = texture_creator.create_texture_from_surface(&surface).expect("texture from surface");
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    let dst = sdl2::rect::Rect::new(x, y, width, height);
    canvas.copy(&texture, None, Some(dst)).ok();
}
