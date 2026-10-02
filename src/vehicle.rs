use std::rc::Rc;

use crate::consts::*;
use crate::geometry::{Direction, Route};
use crate::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleId(pub u64);

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
    pub spawn_time: f64,
    pub min_vel: f64,
    pub max_vel: f64,
    pub color_idx: usize,
    pub granted: bool,
    pub waiting: f64,
}

impl Vehicle {
    pub fn new(id: VehicleId, entry: Direction, route: Route, path: Rc<Path>, sim_time: f64, color_idx: usize) -> Self {
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
            spawn_time: sim_time,
            min_vel: SLOW,
            max_vel: SLOW,
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

        if self.velocity > self.max_vel {
            self.max_vel = self.velocity;
        }
        if self.velocity < self.min_vel {
            self.min_vel = self.velocity;
        }
    }
}
