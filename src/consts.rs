pub const WIN_W: u32 = 900;
pub const WIN_H: u32 = 900;
pub const CENTER: (f64, f64) = (WIN_W as f64 / 2.0, WIN_H as f64 / 2.0);

pub const LANE_W: f64 = 50.0;
pub const APPROACH: f64 = LANE_W * 3.0; // half-width of the road / half-size of the box

pub const BOX_MIN: f64 = CENTER.0 - APPROACH; // == CENTER.1 - APPROACH too (square canvas)
pub const BOX_MAX: f64 = CENTER.0 + APPROACH;

pub const CAR_LEN: f64 = 34.0;
pub const CAR_WID: f64 = 18.0;

pub const SLOW: f64 = 70.0;
pub const MEDIUM: f64 = 130.0;
pub const FAST: f64 = 200.0;

pub const ACCEL: f64 = 140.0; // px/s^2
pub const DECEL: f64 = 220.0; // px/s^2 (braking is stronger than accelerating)

/// Distance the smart-intersection algorithm tries to keep between vehicles.
pub const SAFE_DISTANCE: f64 = 55.0;
/// Any pair of vehicles closer than this (center-to-center) counts as a close call.
pub const CLOSE_CALL_DIST: f64 = 40.0;

pub const GRID_CELL: f64 = 50.0;
/// How far before the intersection box a vehicle eases off FAST to MEDIUM.
pub const CAUTION_ZONE: f64 = 70.0;

pub const MIN_SPAWN_GAP: f64 = 70.0; // don't spawn on top of an existing vehicle in the same lane

pub const RANDOM_SPAWN_INTERVAL: f64 = 0.55;
