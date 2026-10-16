use std::rc::Rc;

use crate::consts::*;
use crate::geometry::{Direction, Route};
use crate::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleId(pub u64);

#[derive(Clone)]
pub struct Vehicle {
    pub id: VehicleId,
    pub entry: Direction,
    pub route: Route,
    pub path: Rc<Path>,
    pub s: f64,
    pub velocity: f64,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    /// Sim time at which the controller first saw this vehicle in its request zone.
    pub detected_at: Option<f64>,
    pub color_idx: usize,
    pub granted: bool,
    pub waiting: f64,
}

impl Vehicle {
    pub fn new(id: VehicleId, entry: Direction, route: Route, path: Rc<Path>, color_idx: usize) -> Self {
        let (x, y, angle) = path.sample(0.0);
        Vehicle {
            id,
            entry,
            route,
            path,
            s: 0.0,
            velocity: SLOW,
            x,
            y,
            angle,
            detected_at: None,
            color_idx,
            granted: false,
            waiting: 0.0,
        }
    }

    pub fn front_s(&self) -> f64 {
        self.s + CAR_LEN / 2.0
    }

    pub fn rear_s(&self) -> f64 {
        self.s - CAR_LEN / 2.0
    }

    /// Corners of the body grown by FOOTPRINT_MARGIN, the area the controller keeps clear.
    pub fn footprint(&self) -> [(f64, f64); 4] {
        let a = self.angle.to_radians();
        let (fx, fy) = (a.sin(), -a.cos());
        let (px, py) = (-fy, fx);
        let hl = CAR_LEN / 2.0 + FOOTPRINT_MARGIN;
        let hw = CAR_WID / 2.0 + FOOTPRINT_MARGIN;
        [(hl, hw), (hl, -hw), (-hl, -hw), (-hl, hw)]
            .map(|(l, w)| (self.x + fx * l + px * w, self.y + fy * l + py * w))
    }

    pub fn is_done(&self) -> bool {
        self.s >= self.path.total_len
    }

    pub fn step(&mut self, target: f64, dt: f64) {
        let target = target.clamp(0.0, FAST);
        if self.velocity < target {
            self.velocity = (self.velocity + ACCEL * dt).min(target);
        } else if self.velocity > target {
            self.velocity = (self.velocity - DECEL * dt).max(target);
        }
        self.velocity = self.velocity.max(0.0);

        self.s = (self.s + self.velocity * dt).min(self.path.total_len);
        let (x, y, angle) = self.path.sample(self.s);
        self.x = x;
        self.y = y;
        self.angle = angle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::PathTable;

    const DT: f64 = 1.0 / 60.0;
    const EPS: f64 = 1e-9;

    fn car(velocity: f64) -> Vehicle {
        let paths = PathTable::new();
        let mut v = Vehicle::new(VehicleId(1), Direction::North, Route::Straight, paths.get(Direction::North, Route::Straight), 0);
        v.velocity = velocity;
        v
    }

    #[test]
    fn acceleration_is_limited() {
        let mut v = car(0.0);
        v.step(FAST, DT);
        assert!((v.velocity - ACCEL * DT).abs() < EPS);
    }

    #[test]
    fn braking_is_limited() {
        let mut v = car(FAST);
        v.step(0.0, DT);
        assert!((v.velocity - (FAST - DECEL * DT)).abs() < EPS);
    }

    #[test]
    fn velocity_settles_on_target_without_overshoot() {
        let mut v = car(MEDIUM - 0.5);
        v.step(MEDIUM, DT);
        assert_eq!(v.velocity, MEDIUM);
        v.step(MEDIUM - 0.5, DT);
        assert_eq!(v.velocity, MEDIUM - 0.5);
    }

    #[test]
    fn target_is_clamped_to_speed_limits() {
        let mut v = car(FAST);
        v.step(FAST * 3.0, DT);
        assert_eq!(v.velocity, FAST);
        let mut v = car(1.0);
        for _ in 0..10 {
            v.step(-50.0, DT);
        }
        assert_eq!(v.velocity, 0.0);
    }

    #[test]
    fn position_advances_by_velocity_and_never_reverses() {
        let mut v = car(MEDIUM);
        v.step(MEDIUM, DT);
        assert!((v.s - MEDIUM * DT).abs() < EPS);
        let mut last = v.s;
        for _ in 0..120 {
            v.step(0.0, DT);
            assert!(v.s >= last);
            last = v.s;
        }
    }

    #[test]
    fn vehicle_finishes_at_end_of_path() {
        let mut v = car(FAST);
        v.s = v.path.total_len - 1.0;
        assert!(!v.is_done());
        v.step(FAST, DT);
        assert_eq!(v.s, v.path.total_len);
        assert!(v.is_done());
    }

    #[test]
    fn footprint_is_body_plus_margin_and_follows_heading() {
        let mut v = car(0.0);
        v.step(0.0, DT);
        let c = v.footprint();
        let side = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
        assert!((side(c[0], c[3]) - (CAR_LEN + 2.0 * FOOTPRINT_MARGIN)).abs() < EPS);
        assert!((side(c[0], c[1]) - (CAR_WID + 2.0 * FOOTPRINT_MARGIN)).abs() < EPS);
        // Car entering from the north drives along y, so the long sides are vertical.
        assert!((c[0].0 - c[3].0).abs() < EPS);
    }
}
