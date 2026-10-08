use std::collections::HashSet;

use crate::consts::CLOSE_CALL_DIST;
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

    /// Summary rows as (label, value, unit).
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
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < CLOSE_CALL_DIST {
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
