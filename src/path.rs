use crate::consts::*;
use crate::geometry::{self, Direction, Route};

#[derive(Clone, Copy)]
struct Sample {
    dist: f64,
    x: f64,
    y: f64,
    angle: f64,
}

pub struct Path {
    samples: Vec<Sample>,
    pub total_len: f64,
    pub box_enter_dist: f64,
    pub box_exit_dist: f64,
    pub cells: Vec<((i32, i32), f64)>,
}

fn angle_from_tangent(dx: f64, dy: f64) -> f64 {
    dx.atan2(-dy).to_degrees()
}

fn unwrap(angles: &mut [f64]) {
    for i in 1..angles.len() {
        let mut d = angles[i] - angles[i - 1];
        while d > 180.0 {
            angles[i] -= 360.0;
            d = angles[i] - angles[i - 1];
        }
        while d < -180.0 {
            angles[i] += 360.0;
            d = angles[i] - angles[i - 1];
        }
    }
}

fn build_from_points(pts: &[(f64, f64)]) -> Path {
    let n = pts.len();
    let mut raw_angles = vec![0.0; n];
    for i in 0..n {
        let (dx, dy) = if i == 0 {
            (pts[1].0 - pts[0].0, pts[1].1 - pts[0].1)
        } else if i == n - 1 {
            (pts[i].0 - pts[i - 1].0, pts[i].1 - pts[i - 1].1)
        } else {
            (pts[i + 1].0 - pts[i - 1].0, pts[i + 1].1 - pts[i - 1].1)
        };
        raw_angles[i] = angle_from_tangent(dx, dy);
    }
    unwrap(&mut raw_angles);

    let mut samples = Vec::with_capacity(n);
    let mut dist = 0.0;
    for i in 0..n {
        if i > 0 {
            let dx = pts[i].0 - pts[i - 1].0;
            let dy = pts[i].1 - pts[i - 1].1;
            dist += (dx * dx + dy * dy).sqrt();
        }
        samples.push(Sample { dist, x: pts[i].0, y: pts[i].1, angle: raw_angles[i] });
    }

    let total_len = dist;
    Path { samples, total_len, box_enter_dist: 0.0, box_exit_dist: total_len, cells: Vec::new() }
}

fn dist_at_point(pts: &[(f64, f64)], target: (f64, f64)) -> f64 {
    let mut d = 0.0;
    let mut best = 0.0;
    let mut best_err = f64::MAX;
    for i in 0..pts.len() {
        if i > 0 {
            let dx = pts[i].0 - pts[i - 1].0;
            let dy = pts[i].1 - pts[i - 1].1;
            d += (dx * dx + dy * dy).sqrt();
        }
        let err = (pts[i].0 - target.0).powi(2) + (pts[i].1 - target.1).powi(2);
        if err < best_err {
            best_err = err;
            best = d;
        }
    }
    best
}

pub fn build_path(entry: Direction, route: Route) -> Path {
    let spawn = geometry::spawn_point(entry, route);
    let exit = geometry::exit_point_final(entry, route);

    if route == Route::Straight {
        let box_enter = geometry::box_entry_point(entry, route);
        let box_exit = geometry::box_exit_point(entry, route);
        let pts = [spawn, box_enter, box_exit, exit];
        let mut path = build_from_points(&pts);
        path.box_enter_dist = dist_at_point(&pts, box_enter);
        path.box_exit_dist = dist_at_point(&pts, box_exit);
        path.cells = swept_cells(&path);
        return path;
    }

    let box_in = geometry::box_entry_point(entry, route);
    let box_out = geometry::box_exit_point(entry, route);

    let control = if entry.is_vertical() { (box_in.0, box_out.1) } else { (box_out.0, box_in.1) };

    let mut pts = Vec::with_capacity(64);
    pts.push(spawn);
    const LEAD: usize = 4;
    for i in 1..LEAD {
        let t = i as f64 / LEAD as f64;
        pts.push((spawn.0 + (box_in.0 - spawn.0) * t, spawn.1 + (box_in.1 - spawn.1) * t));
    }
    const CURVE: usize = 48;
    for i in 0..=CURVE {
        let t = i as f64 / CURVE as f64;
        let mt = 1.0 - t;
        let x = mt * mt * box_in.0 + 2.0 * mt * t * control.0 + t * t * box_out.0;
        let y = mt * mt * box_in.1 + 2.0 * mt * t * control.1 + t * t * box_out.1;
        pts.push((x, y));
    }
    const TAIL: usize = 4;
    for i in 1..=TAIL {
        let t = i as f64 / TAIL as f64;
        pts.push((box_out.0 + (exit.0 - box_out.0) * t, box_out.1 + (exit.1 - box_out.1) * t));
    }

    let mut path = build_from_points(&pts);
    path.box_enter_dist = dist_at_point(&pts, box_in);
    path.box_exit_dist = dist_at_point(&pts, box_out);
    path.cells = swept_cells(&path);
    path
}

fn swept_cells(path: &Path) -> Vec<((i32, i32), f64)> {
    let half_len = CAR_LEN / 2.0 + FOOTPRINT_MARGIN;
    let half_wid = CAR_WID / 2.0 + FOOTPRINT_MARGIN;
    let mut last: std::collections::HashMap<(i32, i32), f64> = std::collections::HashMap::new();
    let mut s = 0.0;
    while s <= path.total_len {
        let (x, y, angle) = path.sample(s);
        let a = angle.to_radians();
        let (fx, fy) = (a.sin(), -a.cos());
        let (px, py) = (-fy, fx);
        const ALONG: usize = 6;
        const ACROSS: usize = 2;
        for i in 0..=ALONG {
            let l = -half_len + 2.0 * half_len * (i as f64 / ALONG as f64);
            for j in 0..=ACROSS {
                let w = -half_wid + 2.0 * half_wid * (j as f64 / ACROSS as f64);
                if let Some(cell) = grid_cell(x + fx * l + px * w, y + fy * l + py * w) {
                    last.insert(cell, s);
                }
            }
        }
        s += 2.0;
    }
    last.into_iter().collect()
}

impl Path {
    pub fn sample(&self, s: f64) -> (f64, f64, f64) {
        let s = s.clamp(0.0, self.total_len);
        let samples = &self.samples;
        let mut lo = 0usize;
        let mut hi = samples.len() - 1;
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if samples[mid].dist <= s {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let a = &samples[lo];
        let b = &samples[hi];
        let span = (b.dist - a.dist).max(1e-9);
        let t = ((s - a.dist) / span).clamp(0.0, 1.0);
        let x = a.x + (b.x - a.x) * t;
        let y = a.y + (b.y - a.y) * t;
        let angle = a.angle + (b.angle - a.angle) * t;
        (x, y, angle)
    }
}

pub struct PathTable {
    paths: std::collections::HashMap<(Direction, Route), std::rc::Rc<Path>>,
}

impl PathTable {
    pub fn new() -> Self {
        let mut paths = std::collections::HashMap::new();
        for &entry in Direction::ALL.iter() {
            for &route in Route::ALL.iter() {
                paths.insert((entry, route), std::rc::Rc::new(build_path(entry, route)));
            }
        }
        PathTable { paths }
    }

    pub fn get(&self, entry: Direction, route: Route) -> std::rc::Rc<Path> {
        self.paths.get(&(entry, route)).unwrap().clone()
    }
}

pub fn grid_cell(x: f64, y: f64) -> Option<(i32, i32)> {
    if x < BOX_MIN || x > BOX_MAX || y < BOX_MIN || y > BOX_MAX {
        return None;
    }
    let cx = ((x - BOX_MIN) / GRID_CELL).floor() as i32;
    let cy = ((y - BOX_MIN) / GRID_CELL).floor() as i32;
    Some((cx, cy))
}
