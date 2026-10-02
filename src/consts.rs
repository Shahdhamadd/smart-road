pub const WIN_W: u32 = 900;
pub const WIN_H: u32 = 900;
pub const CENTER: (f64, f64) = (WIN_W as f64 / 2.0, WIN_H as f64 / 2.0);

pub const LANE_W: f64 = 50.0;
pub const APPROACH: f64 = LANE_W * 3.0;

pub const BOX_MIN: f64 = CENTER.0 - APPROACH;
pub const BOX_MAX: f64 = CENTER.0 + APPROACH;

pub const CAR_LEN: f64 = 34.0;
pub const CAR_WID: f64 = 18.0;

pub const SLOW: f64 = 70.0;
pub const MEDIUM: f64 = 130.0;
pub const FAST: f64 = 200.0;

pub const ACCEL: f64 = 140.0;
pub const DECEL: f64 = 220.0;

pub const SAFE_DISTANCE: f64 = 55.0;
pub const CLOSE_CALL_DIST: f64 = 40.0;

pub const GRID_CELL: f64 = 25.0;
pub const FOOTPRINT_MARGIN: f64 = 5.0;
pub const REQUEST_DIST: f64 = 140.0;
pub const STOP_MARGIN: f64 = 8.0;
pub const CAUTION_ZONE: f64 = 70.0;

pub const MIN_SPAWN_GAP: f64 = CAR_LEN + SAFE_DISTANCE + 16.0;

pub const RANDOM_SPAWN_INTERVAL: f64 = 0.55;
