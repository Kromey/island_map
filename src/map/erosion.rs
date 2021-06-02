//! Simulate hydraulic erosion
//! 
//! This is based on the method described at
//! https://nickmcd.me/2020/04/10/simple-particle-based-hydraulic-erosion/

use super::{elevation::Elevation, SEA_LEVEL};
use nalgebra as na;
use rand::{prelude::*, distributions::Uniform};
use rand_xoshiro::Xoshiro256StarStar;

type Vec2 = na::Vector2<f64>;

/// Time scale for the droplet simulation
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

#[derive(Debug)]
struct Droplet {
    position: Vec2,
    velocity: Vec2,
    volume: f64,
    sediment: f64,
    boundary: f64,
}

impl Droplet {
    fn new(position: Vec2, boundary: u32) -> Self {
        let boundary = (boundary - 1) as f64;

        Self {
            position,
            velocity: Vec2::new(0.0, 0.0),
            volume: 1.0,
            sediment: 0.0,
            boundary,
        }
    }

    fn ipos(&self) -> (u32, u32) {
        (
            self.position[0].clamp(0.0, self.boundary) as u32,
            self.position[1].clamp(0.0, self.boundary) as u32,
        )
    }

    fn speed(&self) -> f64 {
        self.velocity.magnitude()
    }
}

pub fn erode(elevation: &mut Elevation, rng: &mut Xoshiro256StarStar, cycles: u32) {
    let range = Uniform::new(0, elevation.size());

    for _ in 0..cycles {
        // Find a position over our island to drop the Droplet
        let pos = loop {
            let x = range.sample(rng);
            let y = range.sample(rng);
    
            if elevation[(x, y)] > SEA_LEVEL {
                break Vec2::new(x as f64, y as f64);
            }
        };
        let mut drop = Droplet::new(pos, elevation.size());

        while drop.volume > MIN_VOLUME {
            // Floor the position to find the "cell" the droplet is in
            let ipos = drop.ipos();
            // Acceleration vector is the x and y components of the normal vector
            let normal = elevation.get_normal(ipos.0, ipos.1).xy();

            // Model droplet movement via classical physics
            // Accelerate the droplet; F=ma, therefore a=F/m; m=volume*density
            drop.velocity += DT * normal / (drop.volume * DENSITY);
            // Move the droplet
            drop.position += DT * drop.velocity;
            // Slow the droplet via friction
            drop.velocity *= 1.0 - DT * FRICTION;

            // Sedimentation
            // Concentration Equilibrium determines how much sediment a drop can hold
            // Set it higher if drop is faster and moving downhill
            let c_eq = {
                let c_eq = drop.volume * drop.speed() * (elevation[ipos] - elevation[drop.ipos()]);
                c_eq.max(0.0)
            };
            // Compute the driving force (capacity difference)
            let c_diff = c_eq - drop.sediment;
            // Now perform the mass transfer
            drop.sediment += DT * DEPOSITION_RATE * c_diff;
            elevation[ipos] -= DT * drop.volume * DEPOSITION_RATE * c_diff;

            // Remove our droplet if it reaches the ocean
            // This also saves having to do bounds checking on position
            if elevation[ipos] < SEA_LEVEL {
                // Deposit all remaining sediment here
                elevation[ipos] += DT * drop.volume * DEPOSITION_RATE * drop.sediment;

                break;
            }

            // Evaporation
            drop.volume *= 1.0 - DT * EVAP_RATE;
        }
    }
}
