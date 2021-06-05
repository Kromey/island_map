//! Simulate hydraulic erosion
//!
//! This is based on the method described at
//! https://nickmcd.me/2020/04/10/simple-particle-based-hydraulic-erosion/

use super::{elevation::{Elevation, SurfaceType}, SEA_LEVEL};
use nalgebra as na;
use rand::{distributions::Uniform, prelude::*};
use rand_xoshiro::Xoshiro256StarStar;

type Vec2 = na::Vector2<f64>;

/// Time scale ("Delta Time") for the droplet simulation
const DT: f64 = 0.8;
/// Density of a droplet as a function of its volume
/// Altering the density affects inertia of the droplets
const DENSITY: f64 = 1.0;
/// Minimum volume of a droplet, below which it is removed
const MIN_VOLUME: f64 = 0.01;
/// Friction factor
const FRICTION: f64 = 0.05;
/// Evaporation rate
const EVAP_RATE: f64 = 0.001;
/// Rate of deposition of sediment
const DEPOSITION_RATE: f64 = 0.1;
/// Rate of pool filling from a droplet
const VOLUME_FACTOR: f64 = 100.0;
/// Rate of pool draining
const DRAINAGE_RATE: f64 = 0.001;
/// Stream tracking rate
const STREAM_RATE: f64 = 0.01;
/// Stream tracking frequency
const STREAM_UPDATE_FREQUENCY: u32 = 250;

#[derive(Debug)]
struct Droplet {
    position: Vec2,
    velocity: Vec2,
    volume: f64,
    sediment: f64,
}

impl Droplet {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::new(0.0, 0.0),
            volume: 1.0,
            sediment: 0.0,
        }
    }

    fn ipos(&self) -> (u32, u32) {
        (self.position[0] as u32, self.position[1] as u32)
    }

    fn descend(&mut self, elevation: &mut Elevation, track: &mut [bool]) {
        while self.volume > MIN_VOLUME {
            // Floor the position to find the "cell" the droplet is in
            let ipos = self.ipos();
            let idx = elevation.to_idx(ipos.0, ipos.1);

            // Track our position
            track[idx] = true;

            // Get the surface normal to accelerate our droplet
            let normal = elevation.get_normal(ipos.0, ipos.1).xy();

            // Newtonian Mechanics
            // Accelerate the droplet; F=ma, therefore a=F/m; m=volume*density
            let accel = DT * normal / (self.volume * DENSITY);
            self.velocity += accel;
            // Move the droplet
            self.position += DT * self.velocity;
            // Slow the droplet via friction
            self.velocity *= 1.0 - DT * FRICTION;

            // Kill our droplet if it goes out of bounds
            if self
                .position
                .iter()
                .any(|&x| x < 0.0 || x >= elevation.size() as f64)
            {
                // No need to worry about sediment, it's off the map (and, hopefully, in the ocean)
                self.volume = 0.0;
                break;
            }

            // New index
            let nipos = self.ipos();
            let nidx = elevation.to_idx(nipos.0, nipos.1);

            // Surrounded by other droplets and not being accelerated
            if elevation[nidx].stream() > 0.3 && accel.magnitude() < 0.01 {
                break;
            }

            // Entered a pool
            if elevation[nidx].pool() > 0.0 {
                break;
            }

            // Reached the ocean
            if elevation[nidx] < SEA_LEVEL {
                // Deposit all remaining sediment here
                elevation[nidx].add_ground(DT * self.volume * DEPOSITION_RATE * self.sediment);
                // Empty our volume so we don't try to flood here
                self.volume = 0.0;

                break;
            }

            // Sedimentation
            // Concentration Equilibrium determines how much sediment a drop can hold
            // Set it higher if drop is faster and moving downhill
            let c_eq = {
                let c_eq = self.volume
                    * self.velocity.magnitude()
                    * (elevation[idx].ground() - elevation[nidx].ground());
                c_eq.max(0.0)
            };
            // Compute the driving force (capacity difference)
            let c_diff = c_eq - self.sediment;
            // Now perform the mass transfer
            self.sediment += DT * DEPOSITION_RATE * c_diff;
            elevation[idx].remove_ground(DT * self.volume * DEPOSITION_RATE * c_diff);

            // Evaporation
            self.volume *= 1.0 - DT * EVAP_RATE;
            self.sediment /= 1.0 - DT * EVAP_RATE; // Conserve sediment mass
        }
    }

    fn flood(&mut self, elevation: &mut Elevation) {
        // Initialize our starting point
        let ipos = self.ipos();
        let idx = elevation.to_idx(ipos.0, ipos.1);

        let size = elevation.size() as usize;
        
        // Find the pool
        let mut pool = Vec::new();
        let mut active = vec![idx];
        let mut shore = Vec::new();
        let mut visited = vec![false; elevation.len()];

        while let Some(idx) = active.pop() {
            // Make sure we haven't already visited this one
            if visited[idx] {
                continue;
            }
            // Mark it as visited so we don't re-visit
            visited[idx] = true;

            let elev = elevation[idx];
            match elev.surface_type() {
                SurfaceType::Pool => {
                    // Add this to our pool
                    pool.push(idx);

                    // Add neighbors to our active list
                    active.push(idx + size);
                    active.push(idx - size);
                    active.push(idx + 1);
                    active.push(idx - 1);
                    // Diagonal neighbors
                    active.push(idx + size + 1);
                    active.push(idx + size - 1);
                    active.push(idx - size + 1);
                    active.push(idx - size - 1);
                }
                SurfaceType::Ground | SurfaceType::Stream => shore.push((elevation[idx].surface(), idx)),
                SurfaceType::Ocean => unreachable!(), // At least we hope this is unreachable...
            }
        }

        // Now start to incrementally fill our pool until our droplet runs dry or we find a drain
        let mut surface = elevation[idx].surface();
        let mut flooded = vec![false; elevation.len()]; // Track points we've already flooded to
        while self.volume > MIN_VOLUME {
            // Reversed parameters to give us a reversed sorting
            shore.sort_by(|a, b| b.partial_cmp(a).unwrap());

            if let Some((next_surface, idx)) = shore.pop() {
                // Make sure we haven't visited this one, and it's not already a pool
                if flooded[idx] || elevation[idx].surface_type() == SurfaceType::Pool {
                    continue;
                }
                flooded[idx] = true;

                if next_surface < surface {
                    // This is a drain
                    // Move the droplet to this point
                    let (x, y) = elevation.from_idx(idx);
                    self.position = Vec2::new(f64::from(x), f64::from(y));
                    // Remove some sediment
                    self.sediment *= 0.1;

                    // Drain (or rather evaporate) some water from this pool
                    let diff = surface - (1.0 - DRAINAGE_RATE) * surface + DRAINAGE_RATE * next_surface;
                    for idx in pool {
                        elevation[idx].remove_pool(diff);
                    }

                    // Done flooding
                    return;
                } else {
                    // Add this point to our pool
                    let diff = next_surface - surface;
                    // The volume we need to add to the pool from the droplet
                    let new_volume = diff * (pool.len() + 1) as f64 / VOLUME_FACTOR;

                    if new_volume > self.volume {
                        // Can't fill to this point, instead raise our pool as much as we can
                        let added = self.volume / pool.len() as f64 * VOLUME_FACTOR;
                        for idx in pool {
                            elevation[idx].add_pool(added);
                        }
                        // Dry up this droplet and return
                        self.volume = 0.0;
                        return;
                    } else {
                        // Add this point and fill the pool to it
                        pool.push(idx);
                        for &idx in &pool {
                            elevation[idx].add_pool(diff);
                        }
                        // Subtract the used volume from our droplet
                        self.volume -= new_volume;
                        // And raise our testing surface
                        surface = next_surface;

                        // Add neighbors to our shore
                        let new_shore = vec![
                            idx + size,
                            idx - size,
                            idx + 1,
                            idx - 1,
                            idx + size + 1,
                            idx + size - 1,
                            idx - size + 1,
                            idx - size - 1,
                        ];
                        shore.extend(new_shore.into_iter().map(|idx| {
                            (elevation[idx].surface(), idx)
                        }));
                    }
                }
            }
        }
    }
}

pub fn erode(elevation: &mut Elevation, rng: &mut Xoshiro256StarStar, cycles: u32) {
    let range = Uniform::new(0, elevation.size());

    let mut track = vec![false; elevation.len()];

    for i in 0..cycles {
        // Find a position over our island to drop the Droplet
        let pos = loop {
            let x = range.sample(rng);
            let y = range.sample(rng);

            if elevation[(x, y)] > SEA_LEVEL {
                break Vec2::new(f64::from(x), f64::from(y));
            }
        };
        let mut drop = Droplet::new(pos);

        for _ in 0..5 {
            if drop.volume < MIN_VOLUME {
                break;
            }

            drop.descend(elevation, &mut track);

            if drop.volume > MIN_VOLUME {
                drop.flood(elevation);
            }
        }

        if (i + 1) % STREAM_UPDATE_FREQUENCY == 0 {
            for (terrain, tracked) in elevation.iter_mut().zip(track.iter_mut()) {
                terrain.update_stream(*tracked, STREAM_RATE);
                *tracked = false;
            }
        }
    }
}
