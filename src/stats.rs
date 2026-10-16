use std::collections::HashSet;

use crate::consts::{CAR_LEN, SAFE_DISTANCE};
use crate::vehicle::Vehicle;

pub struct Stats {
    pub vehicles_passed: u32,
    pub max_velocity: f64,
    pub min_velocity: f64,
    pub max_time: f64,
    pub min_time: f64,
    pub close_calls: u32,
    pub velocity_sum: f64,
    pub velocity_samples: u64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            vehicles_passed: 0,
            max_velocity: f64::MIN,
            min_velocity: f64::MAX,
            max_time: f64::MIN,
            min_time: f64::MAX,
            close_calls: 0,
            velocity_sum: 0.0,
            velocity_samples: 0,
        }
    }

    pub fn record_velocity_sample(&mut self, v: f64) {
        if v > self.max_velocity {
            self.max_velocity = v;
        }
        if v < self.min_velocity {
            self.min_velocity = v;
        }
        self.velocity_sum += v;
        self.velocity_samples += 1;
    }

    pub fn record_completion(&mut self, travel_time: f64) {
        self.vehicles_passed += 1;
        if travel_time > self.max_time {
            self.max_time = travel_time;
        }
        if travel_time < self.min_time {
            self.min_time = travel_time;
        }
    }

    pub fn average_velocity(&self) -> f64 {
        if self.velocity_samples == 0 {
            0.0
        } else {
            self.velocity_sum / self.velocity_samples as f64
        }
    }

    pub fn rows(&self) -> Vec<(&'static str, String, &'static str)> {
        let fmt = |v: f64| {
            if v.is_finite() && v != f64::MIN && v != f64::MAX {
                format!("{:.1}", v)
            } else {
                "N/A".to_string()
            }
        };
        let avg = if self.velocity_samples == 0 { f64::MIN } else { self.average_velocity() };
        vec![
            ("Vehicles passed", self.vehicles_passed.to_string(), ""),
            ("Avg vehicle speed", fmt(avg), "px/s"),
            ("Max speed", fmt(self.max_velocity), "px/s"),
            ("Min velocity", fmt(self.min_velocity), "px/s"),
            ("Max time to cross", fmt(self.max_time), "s"),
            ("Min time to cross", fmt(self.min_time), "s"),
            ("Close calls", self.close_calls.to_string(), ""),
        ]
    }
}

pub fn update_close_calls(vehicles: &[Vehicle], stats: &mut Stats, active: &mut HashSet<(u64, u64)>) {
    let mut still_active: HashSet<(u64, u64)> = HashSet::new();
    for i in 0..vehicles.len() {
        for j in (i + 1)..vehicles.len() {
            let a = &vehicles[i];
            let b = &vehicles[j];
            if too_close(a, b) {
                let key = if a.id.0 < b.id.0 { (a.id.0, b.id.0) } else { (b.id.0, a.id.0) };
                if !active.contains(&key) {
                    stats.close_calls += 1;
                }
                still_active.insert(key);
            }
        }
    }
    *active = still_active;
}

pub fn too_close(a: &Vehicle, b: &Vehicle) -> bool {
    if a.entry == b.entry && a.route == b.route {
        (a.s - b.s).abs() - CAR_LEN < SAFE_DISTANCE
    } else {
        rects_overlap(&a.footprint(), &b.footprint())
    }
}

fn rects_overlap(a: &[(f64, f64); 4], b: &[(f64, f64); 4]) -> bool {
    for quad in [a, b] {
        for k in 0..2 {
            let (x0, y0) = quad[k];
            let (x1, y1) = quad[k + 1];
            let axis = (y0 - y1, x1 - x0);
            let project = |q: &[(f64, f64); 4]| {
                q.iter().map(|&(x, y)| x * axis.0 + y * axis.1).fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p), hi.max(p)))
            };
            let (alo, ahi) = project(a);
            let (blo, bhi) = project(b);
            if ahi < blo || bhi < alo {
                return false;
            }
        }
    }
    true
}
