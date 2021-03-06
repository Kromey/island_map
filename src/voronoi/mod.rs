use delaunator::{Point, Triangulation};

use fast_poisson::Poisson2D;
use super::river::River;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Coast,
    Lagoon,
    Lake,
    Beach,
}

impl Biome {
    pub fn is_water(&self) -> bool {
        match self {
            Self::Ocean | Self::Coast | Self::Lagoon | Self::Lake => true,
            _ => false,
        }
    }
}

pub struct Cell {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub biome: Biome,
    halfedge: usize,
}

impl Cell {
    pub fn as_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<Point> for Cell {
    fn from(point: Point) -> Cell {
        Cell {
            x: point.x,
            y: point.y,
            height: 0.0,
            biome: Biome::Ocean,
            halfedge: usize::MAX,
        }
    }
}

/// A map represented by Voronoi polygons built from the Delaunay triangulation of random points.
///
/// The implementation of the Voronoi graph from its Delaunay triangulation is based on the article
/// and code at <https://mapbox.github.io/delaunator/>
pub struct Voronoi {
    /// Width of the map
    pub width: u32,
    /// Height of the map
    pub height: u32,
    /// The Voronoi polygon cells
    pub cells: Vec<Cell>,
    /// The Delaunay triangulation of the Voronoi map
    pub delaunay: Triangulation,
    /// Rivers as a series of point indexes
    pub rivers: Vec<River>,
}

impl Voronoi {
    /// Return a new Voronoi map.
    ///
    /// # Arguments
    ///
    /// * `seed` - The RNG seed for generating Voronoi "seeds", i.e. the points around which Voronoi cells are built
    /// * `width` - The width of the map
    /// * `height` - The height of the map
    pub fn new(seed: u64, width: u32, height: u32) -> Voronoi {
        // Generate the seeds from the Poisson disk
        // TODO: The radius should be a parameter exposed to consumers of Voronoi
        let points: Vec<Point> = Poisson2D::new()
            .with_dimensions([f64::from(width), f64::from(height)], 5.0)
            .with_seed(seed)
            .iter()
            .map(|[x, y]| Point { x, y })
            .collect();

        let delaunay = delaunator::triangulate(&points).unwrap();

        // Convert our Vec<Point> into Vec<Cell>
        let mut cells: Vec<Cell> = points
            .into_iter()
            .map(|p| p.into())
            .collect();
        
        let rivers = Vec::new();

        // Build an index of points to an incoming half-edge; useful to find the point's cell
        // and neighbors later
        for e in 0..delaunay.triangles.len() {
            let edge = delaunay.triangles[delaunator::next_halfedge(e)];
            if cells[edge].halfedge == usize::MAX {
                cells[edge].halfedge = e;
            }
        }

        Voronoi {
            width,
            height,
            cells,
            delaunay,
            rivers,
        }
    }

    /// For the specified triangle return its half-edges.
    ///
    /// We define a triangle to be represented by the half-edges that run counter-clockwise around its perimeter.
    ///
    /// # Arguments
    ///
    /// * `triangle` - The index of the triangle to find half-edges for
    pub fn edges_of_triangle(&self, triangle: usize) -> [usize; 3] {
        [3 * triangle, 3 * triangle + 1, 3 * triangle + 2]
    }

    /// Find the triangle the given half-edge belongs to
    ///
    /// # Arguments
    ///
    /// * `halfedge` - The index of the halfedge we want to find the triangle for
    pub fn triangle_of_edge(&self, halfedge: usize) -> usize {
        halfedge / 3
    }

    /// Return the next halfedge
    pub fn next_halfedge(&self, halfedge: usize) -> usize {
        delaunator::next_halfedge(halfedge)
    }

    /// Find the points for the given triangle
    ///
    /// # Arguments
    ///
    /// * `triangle` - The index of the triangle to find points for
    pub fn points_of_triangle(&self, triangle: usize) -> Vec<usize> {
        self.edges_of_triangle(triangle)
            .iter()
            .map(|&e| self.delaunay.triangles[e])
            .collect()
    }

    /// Find the triangles adjacent to the given triangle
    ///
    /// # Arguments
    ///
    /// * `triangle` - The triangle to find neighbors for
    #[allow(dead_code)]
    pub fn adjacent_triangles(&self, triangle: usize) -> Vec<usize> {
        self.edges_of_triangle(triangle)
            .iter()
            .filter_map(|&e| {
                let opposite = self.delaunay.halfedges[e];
                if opposite == delaunator::EMPTY {
                    None
                } else {
                    Some(self.triangle_of_edge(opposite))
                }
            })
            .collect()
    }

    /// Find the circumcenter of the points `a`, `b`, and `c`.
    pub fn circumcenter(&self, a: &Point, b: &Point, c: &Point) -> Point {
        // It's magic math!
        // Actually it's https://en.wikipedia.org/wiki/Circumscribed_circle#Circumcenter_coordinates
        // ...but I don't understand it...
        let ad = a.x.powi(2) + a.y.powi(2);
        let bd = b.x.powi(2) + b.y.powi(2);
        let cd = c.x.powi(2) + c.y.powi(2);

        let d = 2. * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));

        Point {
            x: 1. / d * (ad * (b.y - c.y) + bd * (c.y - a.y) + cd * (a.y - b.y)),
            y: 1. / d * (ad * (c.x - b.x) + bd * (a.x - c.x) + cd * (b.x - a.x)),
        }
    }

    /// Find the circumcenter of the given triangle
    pub fn triangle_center(&self, triangle: usize) -> Point {
        let p = self.points_of_triangle(triangle);
        self.circumcenter(&self.cells[p[0]].as_point(), &self.cells[p[1]].as_point(), &self.cells[p[2]].as_point())
    }

    /// Find the edges that point in to the specified start point.
    ///
    /// # Bugs
    ///
    /// Certain polygons on the convex hull will return incomplete sets of halfedges due to some
    /// triangulations being empty.
    pub fn edges_around_point(&self, start: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut incoming = start;
        loop {
            result.push(incoming);
            let outgoing = self.next_halfedge(incoming);
            incoming = self.delaunay.halfedges[outgoing];

            // TODO: This breaks on certain polygons on the convex hull
            if incoming == delaunator::EMPTY || incoming == start {
                break;
            }
        }

        result
    }

    /// Find the points in polygons that neighbor the given point
    pub fn neighbors_of_point(&self, point: usize) -> Vec<usize> {
        self.edges_around_point(self.cells[point].halfedge)
            .into_iter()
            .map(|edge| self.delaunay.triangles[edge])
            .collect()
    }
}
