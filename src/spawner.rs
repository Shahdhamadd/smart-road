use rand::Rng;

use crate::consts::MIN_SPAWN_GAP;
use crate::geometry::{Direction, Route};
use crate::path::PathTable;
use crate::vehicle::{Vehicle, VehicleId};

/// Tries to create a vehicle entering from `entry` on a randomly chosen route.
/// Refuses (returns None) if the chosen lane already has a vehicle too close to
/// the spawn point, so rapid key-spamming never stacks vehicles on top of each other.
pub fn spawn(
    entry: Direction,
    vehicles: &[Vehicle],
    paths: &PathTable,
    sim_time: f64,
    next_id: &mut u64,
    rng: &mut impl Rng,
    palette_len: usize,
) -> Option<Vehicle> {
    let routes = Route::ALL;
    let start_idx = rng.gen_range(0..routes.len());
    for k in 0..routes.len() {
        let route = routes[(start_idx + k) % routes.len()];
        let blocked = vehicles
            .iter()
            .any(|v| v.entry == entry && v.route == route && v.s < MIN_SPAWN_GAP);
        if !blocked {
            let path = paths.get(entry, route);
            let id = VehicleId(*next_id);
            *next_id += 1;
            let color_idx = rng.gen_range(0..palette_len);
            return Some(Vehicle::new(id, entry, route, path, sim_time, color_idx));
        }
    }
    None
}
