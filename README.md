# Smart Road

A simulation of a traffic-light-free intersection for autonomous vehicles, written in Rust with SDL2. Instead of signals, a central controller adjusts each vehicle's speed so that cars from all four directions cross the junction without colliding.

## Requirements

- Rust (2021 edition) and Cargo
- SDL2 and SDL2_ttf development libraries, discoverable through `pkg-config`

On macOS:

```sh
brew install sdl2 sdl2_ttf pkg-config
```

On Debian/Ubuntu:

```sh
sudo apt install libsdl2-dev libsdl2-ttf-dev pkg-config
```

> The statistics screen loads a system font from hard-coded macOS paths (`FONT_CANDIDATES` in `src/main.rs`). On other platforms the simulation runs, but the statistics text is blank until you add a font path that exists on your system.

## Running

```sh
cargo run --release
```

## Controls

| Key | Action |
| --- | --- |
| `↑` | Spawn a vehicle from the south, travelling north |
| `↓` | Spawn a vehicle from the north, travelling south |
| `→` | Spawn a vehicle from the west, travelling east |
| `←` | Spawn a vehicle from the east, travelling west |
| `R` | Toggle continuous random spawning (one vehicle every 0.55 s) |
| `Esc` | End the simulation and show the statistics screen |

Each spawned vehicle gets a random route (right, straight or left) and a random colour. A spawn is skipped if every lane on that side already has a vehicle too close to the entry point, so holding or spamming a key never stacks cars on top of each other.

On the statistics screen, press `Esc` or `Enter` to close the window.

## The intersection

- 900 × 900 window with two crossing roads, each six lanes wide (three per direction, 50 px per lane).
- Traffic drives on the right. Each lane is dedicated to one route: the outer lane turns right, the middle lane goes straight, and the inner lane turns left.
- A vehicle follows a fixed path for its lane: straight lanes are a single segment, and turns are a quadratic Bézier curve through the 300 × 300 intersection box. All 12 paths are precomputed at startup.

## The smart-intersection algorithm

Every frame, `algorithm::update` picks a target speed for each vehicle, which the vehicle then approaches under acceleration (140 px/s²) and braking (220 px/s²) limits. The target is the minimum of two constraints:

1. **Following distance.** For the nearest vehicle ahead in the same lane, the target is the speed from which the car can still brake to a stop before closing within the safe distance (`v = sqrt(2 · DECEL · d)`). This holds even if the leader stops instantly.
2. **Grid reservation.** The intersection box is divided into a 6 × 6 grid of 50 px cells. Each vehicle occupies the cells under its body and claims the cells just ahead of it. Vehicles are processed in priority order (the one furthest along relative to the box goes first), and a vehicle that finds a cell on its path held by another slows down using the same stopping-speed rule.

Vehicles travel at three speed tiers: they spawn at 70 px/s, cruise at up to 200 px/s on open road, and are capped at 130 px/s from just before the box until they leave it.

## Statistics

When the simulation ends, the following are shown:

- Number of vehicles that passed the intersection
- Max and min velocity reached
- Max and min time taken to cross
- Close calls: pairs of vehicles that came within 40 px of each other, centre to centre (each episode is counted once)
- Average velocity

## Project layout

| File | Purpose |
| --- | --- |
| `src/main.rs` | Window setup, input handling, main loop, statistics screen |
| `src/algorithm.rs` | Smart-intersection controller (speed targets, grid reservation) |
| `src/vehicle.rs` | Vehicle state and physics step |
| `src/path.rs` | Path construction, arc-length sampling, grid cell lookup |
| `src/geometry.rs` | Directions, routes, lane coordinates, entry and exit points |
| `src/spawner.rs` | Vehicle spawning with spawn-gap checks |
| `src/stats.rs` | Statistics collection and close-call tracking |
| `src/render.rs` | Road and vehicle rendering |
| `src/consts.rs` | Tunable constants (sizes, speeds, distances) |

Speeds, safe distances, lane width and the spawn interval can all be adjusted in `src/consts.rs`.
