# Smart Road

A simulation of a traffic-light-free intersection for autonomous vehicles, written in Rust with SDL2. Instead of signals, a central controller adjusts each vehicle's speed so that cars from all four directions cross the junction without colliding.

## Requirements

- Rust (2021 edition) and Cargo
- SDL2 development libraries, discoverable through `pkg-config`

On macOS:

```sh
brew install sdl2 pkg-config
```

On Debian/Ubuntu:

```sh
sudo apt install libsdl2-dev pkg-config
```

> Menu and summary text is drawn with a built-in pixel font (`src/pixel.rs`), so no system fonts are needed.

## Running

```sh
cargo run --release
```

## Controls

The application opens on a menu. Press `Enter` or `Space` (or click **Start**) to start the simulation, or `Esc` to quit.

| Key | Action |
| --- | --- |
| `↑` | Spawn a vehicle from the south, travelling north |
| `↓` | Spawn a vehicle from the north, travelling south |
| `→` | Spawn a vehicle from the west, travelling east |
| `←` | Spawn a vehicle from the east, travelling west |
| `R` | Toggle continuous random spawning (one vehicle every 0.55 s) |
| `Esc` | End the simulation and show the summary screen |

Each spawned vehicle gets a random route (right, straight or left) and a random colour. A spawn is skipped if every lane on that side already has a vehicle too close to the entry point, so holding or spamming a key never stacks cars on top of each other.

On the summary screen, click **Start Again** (or press `Enter`, `Space` or `R`) to run a new simulation, or press `Esc` to quit.

## The intersection

- 900 × 900 window with two crossing roads, each six lanes wide (three per direction, 50 px per lane).
- Traffic drives on the right. Each lane is dedicated to one route: the outer lane turns right, the middle lane goes straight, and the inner lane turns left.
- A vehicle follows a fixed path for its lane: straight lanes are a single segment, and turns are a quadratic Bézier curve through the 300 × 300 intersection box. All 12 paths are precomputed at startup.

## The smart-intersection algorithm

Every frame, `algorithm::update` picks a target speed for each vehicle, which the vehicle then approaches under acceleration (140 px/s²) and braking (220 px/s²) limits. The target is the minimum of two constraints:

1. **Following distance.** For the nearest vehicle ahead in the same lane, the target is the speed from which the car can still brake to a stop before closing within the safe distance (`v = sqrt(2 · a · d)`). This holds even if the leader stops instantly.
2. **Cell reservation.** The intersection box is divided into a 12 × 12 grid of 25 px cells, and every path knows which cells a vehicle's body sweeps on the way through. As a vehicle nears the box it asks for all of those cells at once. If any is held by a vehicle from another lane, it slows down and stops at the line until they are free. Cells are released one by one as the vehicle's body clears them.

Requests are served longest-waiting first, and a waiting vehicle blocks later requests for the cells it needs, so no lane is starved. A vehicle inside the box already owns its whole way out, so vehicles never deadlock.

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
| `src/algorithm.rs` | Smart-intersection controller (speed targets, cell reservation) |
| `src/vehicle.rs` | Vehicle state and physics step |
| `src/path.rs` | Path construction, arc-length sampling, swept grid cells |
| `src/geometry.rs` | Directions, routes, lane coordinates, entry and exit points |
| `src/spawner.rs` | Vehicle spawning with spawn-gap checks |
| `src/stats.rs` | Statistics collection and close-call tracking |
| `src/render.rs` | Road and vehicle rendering |
| `src/consts.rs` | Tunable constants (sizes, speeds, distances) |

Speeds, safe distances, lane width and the spawn interval can all be adjusted in `src/consts.rs`.
