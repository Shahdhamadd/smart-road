use std::collections::HashMap;

use crate::consts::*;
use crate::path::grid_cell;
use crate::vehicle::Vehicle;

/// The smart-intersection controller: for every vehicle picks a target velocity
/// (one of several tiers, reached via acceleration/braking) that (a) keeps a safe
/// following distance behind any vehicle sharing its lane, and (b) prevents two
/// vehicles from entering the same cell of the intersection grid at the same time.
pub fn update(vehicles: &mut [Vehicle], dt: f64) {
    let n = vehicles.len();
    if n == 0 {
        return;
    }

    // Higher-priority vehicles (further along relative to the box) get to claim cells first.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let pa = vehicles[a].s - vehicles[a].path.box_enter_dist;
        let pb = vehicles[b].s - vehicles[b].path.box_enter_dist;
        pb.partial_cmp(&pa).unwrap()
    });

    let mut occupancy: HashMap<(i32, i32), u64> = HashMap::new();
    for v in vehicles.iter() {
        mark_body(v, &mut occupancy);
    }

    let mut targets = vec![0.0f64; n];

    for &i in &order {
        let following_target = {
            let v = &vehicles[i];
            let mut best_gap = f64::MAX;
            for other in vehicles.iter() {
                if other.id == v.id {
                    continue;
                }
                if same_lane(v, other) {
                    let gap = other.rear_s() - v.front_s();
                    if gap >= -CAR_LEN && gap < best_gap {
                        best_gap = gap;
                    }
                }
            }
            if best_gap == f64::MAX {
                FAST
            } else {
                speed_for_gap(best_gap.max(0.0))
            }
        };

        let grid_target = {
            let v = &vehicles[i];
            // Cruise cautiously through the junction itself, even when nothing is
            // blocking: this is one of the (at least 3) distinct speed tiers the
            // smart-intersection controller drives vehicles at.
            let cautious_zone = v.s >= v.path.box_enter_dist - CAUTION_ZONE && v.s <= v.path.box_exit_dist;
            let cap = if cautious_zone { MEDIUM } else { FAST };

            let block_target = if v.s > v.path.box_exit_dist {
                FAST
            } else {
                let lookahead =
                    (v.velocity * 0.5 + v.velocity * v.velocity / (2.0 * DECEL) + 20.0).max(CAR_LEN * 1.5);
                let from_s = v.front_s();
                let to_s = (from_s + lookahead).min(v.path.total_len);
                find_block(v, from_s, to_s, &occupancy)
                    .map(|d| speed_for_block(d - from_s))
                    .unwrap_or(FAST)
            };
            block_target.min(cap)
        };

        let target = following_target.min(grid_target);
        targets[i] = target;

        let v = &vehicles[i];
        if target > 0.0 {
            let reach = v.front_s() + (v.velocity + ACCEL * dt).max(SLOW) * dt * 3.0;
            claim_ahead(v, reach, &mut occupancy);
        }
    }

    for (i, v) in vehicles.iter_mut().enumerate() {
        v.step(targets[i], dt);
    }
}

fn same_lane(a: &Vehicle, b: &Vehicle) -> bool {
    a.entry == b.entry && a.route == b.route
}

/// Target speed for a given bumper-to-bumper gap to the vehicle ahead in the same lane.
///
/// Uses the kinematic "safe approach speed" for the remaining distance
/// (v = sqrt(2 * a * d)) so that, treating the vehicle ahead as if it could stop
/// instantly, this vehicle can always brake (at `DECEL`) to a halt before reaching
/// it. That guarantees no rear-end collision regardless of how abruptly the leader
/// itself has to stop.
fn speed_for_gap(gap: f64) -> f64 {
    let d = (gap - SAFE_DISTANCE).max(0.0);
    (2.0 * DECEL * d).sqrt().min(FAST)
}

/// Target speed given the distance left before entering a grid cell held by another
/// vehicle. Same safe-stopping-speed reasoning as `speed_for_gap`.
fn speed_for_block(dist: f64) -> f64 {
    let d = (dist - SAFE_DISTANCE).max(0.0);
    (2.0 * DECEL * d).sqrt().min(FAST)
}

fn mark_body(v: &Vehicle, occ: &mut HashMap<(i32, i32), u64>) {
    let steps = 6;
    let rear = v.rear_s().max(0.0);
    let front = v.front_s().min(v.path.total_len);
    for i in 0..=steps {
        let s = rear + (front - rear) * (i as f64 / steps as f64);
        let (x, y, _) = v.path.sample(s);
        if let Some(cell) = grid_cell(x, y) {
            occ.insert(cell, v.id.0);
        }
    }
}

fn find_block(v: &Vehicle, from_s: f64, to_s: f64, occ: &HashMap<(i32, i32), u64>) -> Option<f64> {
    if to_s <= from_s {
        return None;
    }
    let step = 6.0;
    let mut s = from_s;
    while s <= to_s {
        let (x, y, _) = v.path.sample(s);
        if let Some(cell) = grid_cell(x, y) {
            if let Some(&owner) = occ.get(&cell) {
                if owner != v.id.0 {
                    return Some(s);
                }
            }
        }
        s += step;
    }
    None
}

fn claim_ahead(v: &Vehicle, reach_s: f64, occ: &mut HashMap<(i32, i32), u64>) {
    let from = v.front_s();
    let to = reach_s.min(v.path.total_len);
    if to <= from {
        return;
    }
    let step = 6.0;
    let mut s = from;
    while s <= to {
        let (x, y, _) = v.path.sample(s);
        if let Some(cell) = grid_cell(x, y) {
            occ.entry(cell).or_insert(v.id.0);
        }
        s += step;
    }
}
