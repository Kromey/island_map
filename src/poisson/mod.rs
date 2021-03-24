//! Generate a Poisson disk distribution.
//!
//! This is an implementation of Bridson's ["Fast Poisson Disk Sampling"][Bridson]
//! implented in 2 dimensions following the description and pseudo-code by
//! [Herman Tulleken][Tulleken].
//! 
//! # Examples
//! 
//! To generate a simple Poisson disk pattern in the range (0, 1] for each of the x and y
//! dimensions:
//! ```
//! let poisson = Pattern::new().generate();
//! ```
//! 
//! To fill a box, specify the width and height (and, optionally, radius):
//! ```
//! let pattern = Pattern {
//!     width: 100.0,
//!     height: 100.0,
//!     radius: 5.0, // Optional, but recommended when width or height are much more than 1.0
//!     ..Default::default()
//! }
//! let poisson = pattern.generate();
//! ```
//! 
//! Note that [`generate`](Pattern::generate) returns an iterator. This allows you to easily map the
//! returned values if you choose:
//! ```
//! # struct Point { x: f64, y: f64 }
//! let points = Pattern::new().generate().map(|(x, y)| Point { x, y });
//! ```
//!
//! [Bridson]: https://www.cct.lsu.edu/~fharhad/ganbatte/siggraph2007/CD2/content/sketches/0250.pdf
//! [Tulleken]: http://devmag.org.za/2009/05/03/poisson-disk-sampling/

use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

/// Builder for a Poisson disk distribution
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Wdith of the box
    pub width: f64,
    /// Height of the box
    pub height: f64,
    /// Radius around each point that must remain empty
    pub radius: f64,
    /// Seed to use for the internal RNG
    pub seed: u64,
    /// Number of samples to generate and test around each point
    pub num_samples: u32,
}

impl Pattern {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> PatternIter {
        PatternIter::new(self)
    }
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            radius: 0.1,
            seed: 0,
            num_samples: 30,
        }
    }
}

/// A Point is simply a two-tuple of f64 values
type Point = (f64, f64);

/// An iterator over the points in the Poisson disk distribution
pub struct PatternIter {
    /// The Pattern from which this iterator was built
    pattern: Pattern,
    /// The RNG
    rng: Xoshiro256StarStar,
    /// The size of each cell in the grid
    cell_size: f64,
    /// The grid stores spatially-oriented samples for fast checking of neighboring sample points
    grid: Vec<Vec<Option<Point>>>,
    /// A list of valid points that we have not yet visited
    active: Vec<Point>,
    /// The current point we are visiting to generate and test surrounding points
    current_sample: Option<(Point, u32)>,
}

impl PatternIter {
    pub fn new(pattern: &Pattern) -> Self {
        // We maintain a grid of our samples for faster radius checking
        let cell_size = pattern.radius / (2_f64).sqrt();

        let mut iter = Self {
            pattern: pattern.clone(),
            rng: Xoshiro256StarStar::seed_from_u64(pattern.seed),
            cell_size,
            grid: vec![vec![None; (pattern.height / cell_size).ceil() as usize]; (pattern.width / cell_size).ceil() as usize],
            active: Vec::new(),
            current_sample: None,
        };
    
        // We have to generate an initial point, just to ensure we've got *something* in the active list
        let first_point = (iter.rng.gen::<f64>() * pattern.width, iter.rng.gen::<f64>() * pattern.height);
        iter.add_point(first_point);

        iter
    }

    /// Add a point to our pattern
    fn add_point(&mut self, point: Point) {
        self.active.push(point);

        // Now stash this point in our grid
        let (x, y) = self.sample_to_grid(point);
        self.grid[x][y] = Some(point);
    }

    /// Convert a sample point into grid cell coordinates
    fn sample_to_grid(&self, point: Point) -> (usize, usize) {
        (
            (point.0 / self.cell_size) as usize,
            (point.1 / self.cell_size) as usize,
        )
    }

    /// Generate a random point between `radius` and `2 * radius` away from the given point
    fn generate_random_point(&mut self) -> Point {
        let point = self.current_sample.unwrap().0;

        let radius = self.pattern.radius * (1.0 + self.rng.gen::<f64>());
        let angle = 2. * std::f64::consts::PI * self.rng.gen::<f64>();
    
        (
            point.0 + radius * angle.cos(),
            point.1 + radius * angle.sin(),
        )
    }
    
    /// Return true if the point is within the bounds of our space.
    ///
    /// This is true if 0 ≤ x < width and 0 ≤ y < height
    fn in_rectangle(&self, point: Point) -> bool {
        point.0 >= 0. && point.0 < self.pattern.width && point.1 >= 0. && point.1 < self.pattern.height
    }
    
    /// Returns true if there is at least one other sample point within `radius` of this point
    fn in_neighborhood(&self, point: Point) -> bool {
        let grid_point = {
            let p = self.sample_to_grid(point);
            (p.0 as isize, p.1 as isize)
        };
        // We'll compare to distance squared, so we can skip the square root operation for better performance
        let r_squared = self.pattern.radius.powi(2);
    
        for x in grid_point.0 - 2..=grid_point.0 + 2 {
            // Make sure we're still in our grid
            if x < 0 || x >= self.grid.len() as isize {
                continue;
            }
            for y in grid_point.1 - 2..=grid_point.1 + 2 {
                // Make sure we're still in our grid
                if y < 0 || y >= self.grid[0].len() as isize {
                    continue;
                }
    
                // If there's a sample here, check that it's not too close to us
                if let Some(point2) = self.grid[x as usize][y as usize] {
                    if (point.0 - point2.0).powi(2) + (point.1 - point2.1).powi(2) < r_squared {
                        return true;
                    }
                }
            }
        }
    
        // We only make it to here if we find no samples too close
        false
    }
}

impl Iterator for PatternIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.current_sample == None {
            //println!("Have no current_sample, getting one from active...");
            if !self.active.is_empty() {
                // Pop points off our active list until it's exhausted
                let point = {
                    let i = self.rng.gen_range(0..self.active.len());
                    self.active.swap_remove(i)
                };
                //println!("Adding {:?} as current sample", point);
                self.current_sample = Some((point, 0));
            }
        }

        if let Some((point, mut i)) = self.current_sample {
            while i < self.pattern.num_samples {
                i += 1;
                self.current_sample = Some((point, i));

                // Generate up to `num_samples` random points between radius and 2*radius from the current point
                let point = self.generate_random_point();
                //println!("Testing new point {:?}", point);
    
                // Ensure we've picked a point inside the bounds of our rectangle, and more than `radius`
                // distance from any other sampled point
                if self.in_rectangle(point)
                    && !self.in_neighborhood(point)
                {
                    //println!("Adding {:?} to active list", point);
                    // We've got a good one!
                    self.add_point(point);

                    return Some(point);
                }
            }

            //println!("Exceeded num_samples, moving to next active sample");
            self.current_sample = None;

            return self.next();
        }

        println!("All done!");
        None
    }
}
