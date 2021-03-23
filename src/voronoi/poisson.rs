use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

/// Number of samples generated/tested around each sampled point.Grid
///
/// Per Bridson, 30 is a "typical" value; Tulleken says it "gives good results". It certainly appears
/// to be suitable for our purposes.
const NUM_SAMPLES: u32 = 30;

/// A Point is simply a two-tuple of f64 values
type Point = (f64, f64);
/// A Grid is a 2-dimensional array of Point values
type Grid = Vec<Vec<Option<Point>>>;

/// Generate a Poisson disk distribution.
///
/// This is an implementation of Bridson's ["Fast Poisson Disk Sampling"][Bridson]
/// implented in 2 dimensions following the description and pseudo-code by
/// [Herman Tulleken][Tulleken].
///
/// # Arguments
///
/// * `width` - Width of the distribution
/// * `height` - Height of the distribution
/// * `radius` - Minimum radius around a sample in which no other sample may appear
/// * `seed` - RNG seed for sampling
///
/// # Returns
///
/// Returns a vector of (f64, f64) tuples representing each sample. The precise number of samples
/// depends upon the width, height, radius, and the RNG's output; while it is deterministic for a
/// given set of all 4 values, the number of samples for a given height, width, and radius will
/// vary with different seeds.
///
/// [Bridson]: https://www.cct.lsu.edu/~fharhad/ganbatte/siggraph2007/CD2/content/sketches/0250.pdf
/// [Tulleken]: http://devmag.org.za/2009/05/03/poisson-disk-sampling/
pub fn generate_poisson(
    width: f64,
    height: f64,
    radius: f64,
    seed: u64,
) -> impl Iterator<Item = Point> {
    let mut rng = Xoshiro256StarStar::seed_from_u64(seed);

    // We maintain a grid of our samples for faster radius checking
    let cell_size = radius / (2f64).sqrt();
    let mut grid =
        vec![vec![None; (height / cell_size).ceil() as usize]; (width / cell_size).ceil() as usize];

    // The active list is samples we have not yet finished evaluating
    let mut active = Vec::new();

    // We have to generate an initial point, just to ensure we've got *something* in the active list
    let first_point = (rng.gen::<f64>() * width, rng.gen::<f64>() * height);
    active.push(first_point);
    // And of course we put it in our grid, too
    let (x, y) = sample_to_grid(first_point, cell_size);
    grid[x][y] = Some(first_point);

    while !active.is_empty() {
        // Pop points off our active list until it's exhausted
        let point = active.pop().expect("Already confirmed vec is not empty");

        for _ in 0..NUM_SAMPLES {
            // Generate up to NUM_SAMPLES random points between radius and 2*radius from the current point
            let point = generate_random_point_around(point, radius, &mut rng);

            // Ensure we've picked a point inside the bounds of our rectangle, and more than `radius`
            // distance from any other sampled point
            if in_rectangle(point, width, height)
                && !in_neighborhood(point, &grid, radius, cell_size)
            {
                // We've got a good one! Add to our active list
                active.push(point);

                // And add it to our grid too
                let grid_point = sample_to_grid(point, cell_size);
                grid[grid_point.0][grid_point.1] = Some(point);
            }
        }
    }

    // While it would be faster of course to store an additional Vec with our samples as we find
    // them, this is plenty fast. Also by skipping the `.collect()` call and returning the iterator
    // itself, we can let calling code choose to do further iteration (e.g. to map into a different
    // type) or just `.collect()` the samples directly.
    // Note that we lose any semblance of randomness to the order that points are returned this way,
    // but I doubt it's a problem and a samples Vec would still have a general trend of spreading
    // outward from the initial point anyway, rather than being truly random.
    grid.into_iter().flatten().filter_map(|point| point)
}

/// Convert a sample point into grid cell coordinates
fn sample_to_grid(point: Point, cell_size: f64) -> (usize, usize) {
    (
        (point.0 / cell_size) as usize,
        (point.1 / cell_size) as usize,
    )
}

/// Generate a random point between `radius` and `2 * radius` away from the given point
fn generate_random_point_around(point: Point, radius: f64, rng: &mut Xoshiro256StarStar) -> Point {
    let radius = radius * (1.0 + rng.gen::<f64>());
    let angle = 2. * std::f64::consts::PI * rng.gen::<f64>();

    (
        point.0 + radius * angle.cos(),
        point.1 + radius * angle.sin(),
    )
}

/// Return true if the point is within the bounds of our space.
///
/// This is true if 0 ≤ x < width and 0 ≤ y < height
fn in_rectangle(point: Point, width: f64, height: f64) -> bool {
    point.0 >= 0. && point.0 < width && point.1 >= 0. && point.1 < height
}

/// Returns true if there is at least one other sample point within `radius` of this point
fn in_neighborhood(point: Point, grid: &Grid, radius: f64, cell_size: f64) -> bool {
    let grid_point = {
        let p = sample_to_grid(point, cell_size);
        (p.0 as isize, p.1 as isize)
    };
    // We'll compare to distance squared, so we can skip the square root operation for better performance
    let r_squared = radius.powi(2);

    for x in grid_point.0 - 2..=grid_point.0 + 2 {
        // Make sure we're still in our grid
        if x < 0 || x >= grid.len() as isize {
            continue;
        }
        for y in grid_point.1 - 2..=grid_point.1 + 2 {
            // Make sure we're still in our grid
            if y < 0 || y >= grid[0].len() as isize {
                continue;
            }

            // If there's a sample here, check that it's not too close to us
            if let Some(point2) = grid[x as usize][y as usize] {
                if (point.0 - point2.0).powi(2) + (point.1 - point2.1).powi(2) < r_squared {
                    return true;
                }
            }
        }
    }

    // We only make it to here if we find no samples too close
    false
}
