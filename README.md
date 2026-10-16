# Smart Road

A simulation of a traffic-light-free intersection for autonomous vehicles, written in Rust with SDL2. Instead of signals, a central controller adjusts each vehicle's speed so that cars from all four directions cross the junction without colliding.

## Requirements

- Rust (2021 edition) and Cargo
- macOS/Linux: SDL2 development libraries, discoverable through `pkg-config`
- Windows: CMake and a C compiler (SDL2 itself is built from source automatically)

On macOS:

```sh
brew install sdl2 pkg-config
```

On Debian/Ubuntu:

```sh
sudo apt install libsdl2-dev pkg-config
```

On Windows, no SDL2 install is needed: Cargo builds SDL2 from source and links it statically into the executable (the `bundled` and `static-link` features of the `sdl2` crate, enabled only for Windows in `Cargo.toml`). You need:

1. Rust with the default MSVC toolchain (`x86_64-pc-windows-msvc`), installed via [rustup](https://rustup.rs). Its installer offers the Visual Studio C++ Build Tools, which provide the C compiler.
2. CMake on your `PATH`:

   ```powershell
   winget install Kitware.CMake
   ```

   Open a new terminal afterwards so the updated `PATH` is picked up.

The first build takes a few minutes longer while SDL2 compiles; later builds are cached. The resulting `smart_road.exe` runs on its own, with no `SDL2.dll` next to it.

> Menu and summary text is drawn with a built-in pixel font (`src/pixel.rs`), so no system fonts are needed.

## Running

```sh
cargo run --release
```

`cargo test --release` runs the test suite. No window is opened, because the controller and the vehicle physics don't depend on SDL2.

- `src/vehicle.rs`: acceleration and braking limits, the speed cap, position updates and footprint shape.
- `src/algorithm.rs`: safe following distance (including a leader that stops instantly), cell reservation (crossing lanes are never granted together, cells are released once a vehicle has passed them, longest-waiting vehicle first, a waiting vehicle holds its place in the queue), stopping before the box, and one vehicle per lane all crossing with no close calls.
- `src/main.rs`: several minutes of heavy random traffic with no close calls, and the grid boundary.

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

## Summary screen

Pressing `Esc` during the simulation ends the run and shows a summary:

| Stat | Meaning |
| --- | --- |
| Vehicles passed | Number of vehicles that crossed the intersection and left the window |
| Avg vehicle speed | Average velocity of all vehicles over the whole run |
| Max speed | Highest velocity any vehicle reached |
| Min velocity | Lowest velocity any vehicle reached (0 if a vehicle had to stop) |
| Max time to cross | Longest time a vehicle took from detection to leaving the window |
| Min time to cross | Shortest time a vehicle took from detection to leaving the window |
| Close calls | Pairs of vehicles that broke the controller's spacing rules (each episode is counted once, see below) |

A vehicle is **detected** when its front reaches the request zone, 140 px before the intersection box, where the controller first starts managing it. Time spent driving in from the window edge before that is not counted.

A **close call** is any moment two vehicles break the spacing the controller is meant to guarantee:

- **Same lane:** the bumper-to-bumper gap drops below the safe distance (55 px).
- **Different lanes:** their footprints overlap. A footprint is the car body plus a 5 px margin on every side, the same area the cell reservation keeps clear.

Speeds are in px/s and times in seconds. A stat shows `N/A` if there is no data for it yet, for example the crossing times when no vehicle has finished. The close-calls marker is green when the count is zero and red otherwise.

**Start Again** runs a fresh simulation with every stat reset, without going back to the menu.

## Look and feel

The menu and summary screens are drawn in a pixel-art style: a built-in 5 × 7 bitmap font, panels with stepped corners and hard drop shadows, and blocky buttons that brighten on hover. Everything is drawn with plain SDL2 rectangles, so there are no font or image assets to ship.

## Project layout

| File | Purpose |
| --- | --- |
| `src/main.rs` | Window setup, input handling, simulation loop, menu and summary screens |
| `src/algorithm.rs` | Smart-intersection controller (speed targets, cell reservation) |
| `src/vehicle.rs` | Vehicle state and physics step |
| `src/path.rs` | Path construction, arc-length sampling, swept grid cells |
| `src/geometry.rs` | Directions, routes, lane coordinates, entry and exit points |
| `src/spawner.rs` | Vehicle spawning with spawn-gap checks |
| `src/stats.rs` | Statistics collection and close-call tracking |
| `src/render.rs` | Road and vehicle rendering |
| `src/pixel.rs` | Pixel font and pixel-style panels and buttons for the menu and summary |
| `src/consts.rs` | Tunable constants (sizes, speeds, distances) |

Speeds, safe distances, lane width and the spawn interval can all be adjusted in `src/consts.rs`.
