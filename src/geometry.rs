use crate::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North, // vehicle enters from the north edge, travels south
    South, // enters from the south edge, travels north
    East,  // enters from the east edge, travels west
    West,  // enters from the west edge, travels east
}

impl Direction {
    pub const ALL: [Direction; 4] = [Direction::North, Direction::South, Direction::East, Direction::West];

    /// True when travel is along the vertical (y) axis for this entry.
    pub fn is_vertical(self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Right,
    Straight,
    Left,
}

impl Route {
    pub const ALL: [Route; 3] = [Route::Right, Route::Straight, Route::Left];
}

/// x-coordinate of the lane for a North/South entry.
fn lane_x(entry: Direction, route: Route) -> f64 {
    match (entry, route) {
        (Direction::North, Route::Right) => CENTER.0 - 125.0,
        (Direction::North, Route::Straight) => CENTER.0 - 75.0,
        (Direction::North, Route::Left) => CENTER.0 - 25.0,
        (Direction::South, Route::Left) => CENTER.0 + 25.0,
        (Direction::South, Route::Straight) => CENTER.0 + 75.0,
        (Direction::South, Route::Right) => CENTER.0 + 125.0,
        _ => unreachable!("lane_x only valid for N/S entries"),
    }
}

/// y-coordinate of the lane for an East/West entry.
fn lane_y(entry: Direction, route: Route) -> f64 {
    match (entry, route) {
        (Direction::West, Route::Left) => CENTER.1 + 25.0,
        (Direction::West, Route::Straight) => CENTER.1 + 75.0,
        (Direction::West, Route::Right) => CENTER.1 + 125.0,
        (Direction::East, Route::Right) => CENTER.1 - 125.0,
        (Direction::East, Route::Straight) => CENTER.1 - 75.0,
        (Direction::East, Route::Left) => CENTER.1 - 25.0,
        _ => unreachable!("lane_y only valid for E/W entries"),
    }
}

/// The lane coordinate (x for N/S entries, y for E/W entries) a vehicle drives on.
pub fn lane_coord(entry: Direction, route: Route) -> f64 {
    if entry.is_vertical() {
        lane_x(entry, route)
    } else {
        lane_y(entry, route)
    }
}

/// Destination direction+role used to find the exit lane coordinate for a turn.
fn destination(entry: Direction, route: Route) -> (Direction, Route) {
    match (entry, route) {
        (Direction::North, Route::Right) => (Direction::East, Route::Right),
        (Direction::North, Route::Left) => (Direction::West, Route::Left),
        (Direction::South, Route::Right) => (Direction::West, Route::Right),
        (Direction::South, Route::Left) => (Direction::East, Route::Left),
        (Direction::West, Route::Right) => (Direction::North, Route::Right),
        (Direction::West, Route::Left) => (Direction::South, Route::Left),
        (Direction::East, Route::Right) => (Direction::South, Route::Right),
        (Direction::East, Route::Left) => (Direction::North, Route::Left),
        (_, Route::Straight) => (entry, Route::Straight),
    }
}

/// Coordinate of the exit lane (x for a vertical exit road, y for a horizontal one).
pub fn exit_lane_coord(entry: Direction, route: Route) -> f64 {
    let (dest_dir, dest_route) = destination(entry, route);
    lane_coord(dest_dir, dest_route)
}

pub fn spawn_point(entry: Direction, route: Route) -> (f64, f64) {
    let start_off = -CAR_LEN; // fully off-canvas
    match entry {
        Direction::North => (lane_x(entry, route), start_off),
        Direction::South => (lane_x(entry, route), WIN_H as f64 - start_off),
        Direction::West => (start_off, lane_y(entry, route)),
        Direction::East => (WIN_W as f64 - start_off, lane_y(entry, route)),
    }
}

pub fn exit_point_final(entry: Direction, route: Route) -> (f64, f64) {
    let (dest_dir, _) = destination(entry, route);
    let end_off = WIN_W as f64 + CAR_LEN;
    let coord = exit_lane_coord(entry, route);
    match dest_dir {
        Direction::South => (coord, end_off),
        Direction::North => (coord, -CAR_LEN),
        Direction::East => (-CAR_LEN, coord),
        Direction::West => (end_off, coord),
    }
}

/// Point where the entry lane crosses the near boundary of the intersection box.
pub fn box_entry_point(entry: Direction, route: Route) -> (f64, f64) {
    match entry {
        Direction::North => (lane_x(entry, route), BOX_MIN),
        Direction::South => (lane_x(entry, route), BOX_MAX),
        Direction::West => (BOX_MIN, lane_y(entry, route)),
        Direction::East => (BOX_MAX, lane_y(entry, route)),
    }
}

/// Point where the exit lane crosses the far boundary of the intersection box.
pub fn box_exit_point(entry: Direction, route: Route) -> (f64, f64) {
    let (dest_dir, _) = destination(entry, route);
    let coord = exit_lane_coord(entry, route);
    match dest_dir {
        Direction::South => (coord, BOX_MAX),
        Direction::North => (coord, BOX_MIN),
        Direction::East => (BOX_MIN, coord),
        Direction::West => (BOX_MAX, coord),
    }
}
