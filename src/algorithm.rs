use std::collections::{HashMap, HashSet};

use crate::consts::*;
use crate::geometry::{Direction, Route};
use crate::vehicle::Vehicle;

pub fn update(vehicles: &mut [Vehicle], dt: f64) {
    grant_reservations(vehicles, dt);

    let targets: Vec<f64> = vehicles
        .iter()
        .map(|v| following_target(v, vehicles).min(box_target(v)))
        .collect();

    for (v, target) in vehicles.iter_mut().zip(targets) {
        v.step(target, dt);
    }
}

fn grant_reservations(vehicles: &mut [Vehicle], dt: f64) {
    let mut held: HashMap<(i32, i32), (Direction, Route)> = HashMap::new();
    for v in vehicles.iter().filter(|v| v.granted) {
        for &(cell, last_s) in v.path.cells.iter() {
            if v.s <= last_s {
                held.insert(cell, (v.entry, v.route));
            }
        }
    }

    let mut queue: Vec<usize> = Vec::new();
    for (i, v) in vehicles.iter_mut().enumerate() {
        if !v.granted && v.front_s() >= v.path.box_enter_dist - REQUEST_DIST {
            v.waiting += dt;
            queue.push(i);
        }
    }
    queue.sort_by(|&a, &b| {
        let (va, vb) = (&vehicles[a], &vehicles[b]);
        vb.waiting.partial_cmp(&va.waiting).unwrap().then(va.id.0.cmp(&vb.id.0))
    });

    let mut pending: HashSet<(i32, i32)> = HashSet::new();
    for i in queue {
        let v = &mut vehicles[i];
        let lane = (v.entry, v.route);
        let free = v
            .path
            .cells
            .iter()
            .all(|(cell, _)| !pending.contains(cell) && held.get(cell).map_or(true, |&l| l == lane));
        if free {
            v.granted = true;
            for &(cell, _) in v.path.cells.iter() {
                held.insert(cell, lane);
            }
        } else {
            pending.extend(v.path.cells.iter().map(|&(cell, _)| cell));
        }
    }
}

fn following_target(v: &Vehicle, vehicles: &[Vehicle]) -> f64 {
    vehicles
        .iter()
        .filter(|other| other.id != v.id && other.entry == v.entry && other.route == v.route && other.s > v.s)
        .map(|other| other.rear_s() - v.front_s())
        .fold(None, |best: Option<f64>, gap| Some(best.map_or(gap, |b| b.min(gap))))
        .map_or(FAST, |gap| safe_speed(gap - SAFE_DISTANCE))
}

fn box_target(v: &Vehicle) -> f64 {
    let cautious = v.s >= v.path.box_enter_dist - CAUTION_ZONE && v.s <= v.path.box_exit_dist;
    let cap = if cautious { MEDIUM } else { FAST };
    if v.granted {
        cap
    } else {
        safe_speed(v.path.box_enter_dist - STOP_MARGIN - v.front_s()).min(cap)
    }
}

fn safe_speed(dist: f64) -> f64 {
    (2.0 * 0.8 * DECEL * dist.max(0.0)).sqrt().min(FAST)
}
