use std::collections::{HashMap, HashSet};

use crate::consts::*;
use crate::geometry::{Direction, Route};
use crate::vehicle::Vehicle;

pub fn update(vehicles: &mut [Vehicle], dt: f64, sim_time: f64) {
    grant_reservations(vehicles, dt, sim_time);

    let targets: Vec<f64> = vehicles
        .iter()
        .map(|v| following_target(v, vehicles).min(box_target(v)))
        .collect();

    for (v, target) in vehicles.iter_mut().zip(targets) {
        v.step(target, dt);
    }
}

fn grant_reservations(vehicles: &mut [Vehicle], dt: f64, sim_time: f64) {
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
            v.detected_at.get_or_insert(sim_time);
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
        .map_or(FAST, |gap| safe_speed(gap - SAFE_DISTANCE - FOLLOW_BUFFER))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::PathTable;
    use crate::stats::too_close;
    use crate::vehicle::VehicleId;

    const DT: f64 = 1.0 / 60.0;

    fn car(paths: &PathTable, id: u64, entry: Direction, route: Route, s: f64, velocity: f64) -> Vehicle {
        let mut v = Vehicle::new(VehicleId(id), entry, route, paths.get(entry, route), 0);
        v.velocity = velocity;
        place(&mut v, s);
        v
    }

    fn place(v: &mut Vehicle, s: f64) {
        v.s = s;
        (v.x, v.y, v.angle) = v.path.sample(s);
    }

    fn in_request_zone(paths: &PathTable, entry: Direction, route: Route) -> f64 {
        paths.get(entry, route).box_enter_dist - REQUEST_DIST + 1.0 - CAR_LEN / 2.0
    }

    fn cells(paths: &PathTable, lane: (Direction, Route)) -> HashSet<(i32, i32)> {
        paths.get(lane.0, lane.1).cells.iter().map(|&(c, _)| c).collect()
    }

    fn conflict(paths: &PathTable, a: (Direction, Route), b: (Direction, Route)) -> bool {
        !cells(paths, a).is_disjoint(&cells(paths, b))
    }

    fn all_lanes() -> Vec<(Direction, Route)> {
        Direction::ALL.iter().flat_map(|&d| Route::ALL.iter().map(move |&r| (d, r))).collect()
    }

    fn gap(follower: &Vehicle, leader: &Vehicle) -> f64 {
        leader.rear_s() - follower.front_s()
    }

    #[test]
    fn safe_speed_can_always_stop_within_the_distance() {
        assert_eq!(safe_speed(0.0), 0.0);
        assert_eq!(safe_speed(-10.0), 0.0);
        assert_eq!(safe_speed(1e6), FAST);
        for d in [0.5, 1.0, 5.0, 20.0, 50.0, 100.0] {
            let v = safe_speed(d);
            assert!(v * v / (2.0 * DECEL) <= d, "at {d} px, {v} px/s cannot stop in time");
        }
    }

    #[test]
    fn free_road_and_other_lanes_do_not_slow_a_vehicle() {
        let paths = PathTable::new();
        let v = car(&paths, 1, Direction::North, Route::Straight, 50.0, FAST);
        let other_lane = car(&paths, 2, Direction::North, Route::Left, 60.0, 0.0);
        let behind = car(&paths, 3, Direction::North, Route::Straight, 0.0, 0.0);
        assert_eq!(following_target(&v, &[v.clone(), other_lane, behind]), FAST);
    }

    #[test]
    fn follower_stops_just_behind_a_stopped_leader() {
        let paths = PathTable::new();
        let leader = car(&paths, 1, Direction::North, Route::Straight, 600.0, 0.0);
        let mut f = car(&paths, 2, Direction::North, Route::Straight, 100.0, FAST);
        for _ in 0..60 * 20 {
            let target = following_target(&f, &[leader.clone(), f.clone()]);
            f.step(target, DT);
            assert!(gap(&f, &leader) >= SAFE_DISTANCE, "gap fell to {:.3}", gap(&f, &leader));
        }
        assert_eq!(f.velocity, 0.0);
        assert!(gap(&f, &leader) <= SAFE_DISTANCE + FOLLOW_BUFFER + 1.0, "stopped too far back: {:.3}", gap(&f, &leader));
    }

    #[test]
    fn follower_keeps_safe_distance_when_leader_stops_instantly() {
        let paths = PathTable::new();
        let start_gap = SAFE_DISTANCE + FOLLOW_BUFFER + FAST * FAST / (2.0 * 0.8 * DECEL);
        let mut leader = car(&paths, 1, Direction::East, Route::Straight, 500.0, FAST);
        let mut f = car(&paths, 2, Direction::East, Route::Straight, 500.0 - CAR_LEN - start_gap, FAST);
        assert_eq!(following_target(&f, &[leader.clone(), f.clone()]), FAST);

        leader.velocity = 0.0;
        for _ in 0..60 * 5 {
            let target = following_target(&f, &[leader.clone(), f.clone()]);
            f.step(target, DT);
            assert!(gap(&f, &leader) >= SAFE_DISTANCE, "gap fell to {:.3}", gap(&f, &leader));
        }
        assert_eq!(f.velocity, 0.0);
    }

    #[test]
    fn crossing_lanes_are_never_granted_together() {
        let paths = PathTable::new();
        let (a, b) = ((Direction::North, Route::Straight), (Direction::East, Route::Straight));
        assert!(conflict(&paths, a, b));
        let mut vs = vec![
            car(&paths, 1, a.0, a.1, in_request_zone(&paths, a.0, a.1), MEDIUM),
            car(&paths, 2, b.0, b.1, in_request_zone(&paths, b.0, b.1), MEDIUM),
        ];
        grant_reservations(&mut vs, DT, 0.0);
        assert_eq!(vs.iter().filter(|v| v.granted).count(), 1);
    }

    #[test]
    fn disjoint_lanes_are_granted_together() {
        let paths = PathTable::new();
        let (a, b) = ((Direction::North, Route::Right), (Direction::South, Route::Right));
        assert!(!conflict(&paths, a, b));
        let mut vs = vec![
            car(&paths, 1, a.0, a.1, in_request_zone(&paths, a.0, a.1), MEDIUM),
            car(&paths, 2, b.0, b.1, in_request_zone(&paths, b.0, b.1), MEDIUM),
        ];
        grant_reservations(&mut vs, DT, 0.0);
        assert!(vs.iter().all(|v| v.granted));
    }

    #[test]
    fn same_lane_vehicles_share_their_cells() {
        let paths = PathTable::new();
        let (d, r) = (Direction::West, Route::Left);
        let zone = in_request_zone(&paths, d, r);
        let mut vs = vec![car(&paths, 1, d, r, zone + 60.0, MEDIUM), car(&paths, 2, d, r, zone, MEDIUM)];
        grant_reservations(&mut vs, DT, 0.0);
        assert!(vs.iter().all(|v| v.granted));
    }

    #[test]
    fn vehicles_outside_the_request_zone_are_not_detected() {
        let paths = PathTable::new();
        let (d, r) = (Direction::South, Route::Straight);
        let mut vs = vec![car(&paths, 1, d, r, in_request_zone(&paths, d, r) - 2.0, MEDIUM)];
        grant_reservations(&mut vs, DT, 3.0);
        assert!(!vs[0].granted);
        assert_eq!(vs[0].detected_at, None);
        let s = vs[0].s + 2.0;
        place(&mut vs[0], s);
        grant_reservations(&mut vs, DT, 4.0);
        assert!(vs[0].granted);
        assert_eq!(vs[0].detected_at, Some(4.0));
    }

    #[test]
    fn longest_waiting_vehicle_is_served_first() {
        let paths = PathTable::new();
        let (a, b) = ((Direction::North, Route::Straight), (Direction::West, Route::Straight));
        assert!(conflict(&paths, a, b));
        let mut vs = vec![
            car(&paths, 1, a.0, a.1, in_request_zone(&paths, a.0, a.1), 0.0),
            car(&paths, 2, b.0, b.1, in_request_zone(&paths, b.0, b.1), 0.0),
        ];
        vs[1].waiting = 5.0;
        grant_reservations(&mut vs, DT, 0.0);
        assert!(!vs[0].granted && vs[1].granted);
    }

    #[test]
    fn a_waiting_vehicle_blocks_later_requests_for_its_cells() {
        let paths = PathTable::new();
        let lanes = all_lanes();
        let lanes = &lanes;
        let (x, y, z) = lanes
            .iter()
            .flat_map(|&x| lanes.iter().flat_map(move |&y| lanes.iter().map(move |&z| (x, y, z))))
            .find(|&(x, y, z)| conflict(&paths, x, y) && conflict(&paths, y, z) && !conflict(&paths, x, z) && x != y && y != z)
            .expect("some lane triple chains conflicts");

        let mut holder = car(&paths, 1, x.0, x.1, paths.get(x.0, x.1).box_enter_dist, MEDIUM);
        holder.granted = true;
        let mut waiter = car(&paths, 2, y.0, y.1, in_request_zone(&paths, y.0, y.1), 0.0);
        waiter.waiting = 3.0;
        let newcomer = car(&paths, 3, z.0, z.1, in_request_zone(&paths, z.0, z.1), MEDIUM);
        let mut vs = vec![holder, waiter, newcomer];

        grant_reservations(&mut vs, DT, 0.0);
        assert!(!vs[1].granted, "waiter should still be blocked by the holder");
        assert!(!vs[2].granted, "newcomer jumped the queue");
    }

    #[test]
    fn cells_are_released_once_the_vehicle_has_passed_them() {
        let paths = PathTable::new();
        let (a, b) = ((Direction::North, Route::Straight), (Direction::East, Route::Straight));
        let path_a = paths.get(a.0, a.1);
        let cleared = path_a.cells.iter().map(|&(_, last_s)| last_s).fold(0.0, f64::max) + 1.0;

        let mut holder = car(&paths, 1, a.0, a.1, path_a.box_enter_dist, MEDIUM);
        holder.granted = true;
        let mut vs = vec![holder, car(&paths, 2, b.0, b.1, in_request_zone(&paths, b.0, b.1), MEDIUM)];
        grant_reservations(&mut vs, DT, 0.0);
        assert!(!vs[1].granted);

        place(&mut vs[0], cleared);
        grant_reservations(&mut vs, DT, 0.0);
        assert!(vs[1].granted);
    }

    #[test]
    fn ungranted_vehicle_stops_before_the_box() {
        let paths = PathTable::new();
        let (d, r) = (Direction::South, Route::Left);
        let v = car(&paths, 1, d, r, 0.0, 0.0);
        let enter = v.path.box_enter_dist;
        for front in [enter - 200.0, enter - 50.0, enter - STOP_MARGIN, enter] {
            let mut probe = car(&paths, 1, d, r, front - CAR_LEN / 2.0, 0.0);
            probe.granted = false;
            let target = box_target(&probe);
            let stop = target * target / (2.0 * DECEL);
            assert!(front + stop <= enter, "front {front:.0} at {target:.1} px/s overruns the box");
        }
    }

    #[test]
    fn one_vehicle_per_lane_all_cross_without_close_calls() {
        let paths = PathTable::new();
        let mut vs: Vec<Vehicle> = all_lanes()
            .into_iter()
            .enumerate()
            .map(|(i, (d, r))| car(&paths, i as u64, d, r, 0.0, SLOW))
            .collect();
        let mut t = 0.0;
        while !vs.is_empty() {
            t += DT;
            assert!(t < 60.0, "{} vehicles still stuck after 60 s", vs.len());
            update(&mut vs, DT, t);
            for v in &vs {
                if v.front_s() > v.path.box_enter_dist {
                    assert!(v.granted, "vehicle {} entered the box without a reservation", v.id.0);
                }
            }
            for i in 0..vs.len() {
                for j in (i + 1)..vs.len() {
                    assert!(!too_close(&vs[i], &vs[j]), "vehicles {} and {} too close", vs[i].id.0, vs[j].id.0);
                }
            }
            vs.retain(|v| !v.is_done());
        }
    }
}
