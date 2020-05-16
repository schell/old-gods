use specs::prelude::*;

use super::super::prelude::{AABB, V2};

// TODO: Real SAT for polygons, AABB and circles.
// See http://www.metanetsoftware.com/2016/n-tutorial-a-collision-detection-and-response

#[derive(Debug, Clone, PartialEq)]
/// A number of different shapes. A shape itself doesn't have a position, it
/// only describes the dimensions and separating axes.
pub enum Shape {
    /// A box defined by two points.
    Box {
        lower: V2, // top left
        upper: V2, // bottom right
    },

    /// An assumed to be convex polygon
    Polygon { vertices: Vec<V2> },
}


impl Shape {
    pub fn box_with_size(w: f32, h: f32) -> Shape {
        Shape::Box {
            upper: V2::origin(),
            lower: V2::new(w, h),
        }
    }

    /// The axis aligned box needed to contain the shape
    pub fn aabb(&self) -> AABB {
        match &self {
            Shape::Box { lower, upper } => AABB::from_points(*lower, *upper),
            Shape::Polygon { vertices } => {
                let mut left = std::f32::INFINITY;
                let mut right = std::f32::NEG_INFINITY;
                let mut top = std::f32::INFINITY;
                let mut bottom = std::f32::NEG_INFINITY;

                for v in vertices {
                    if v.x < left {
                        left = v.x;
                    }

                    if v.x > right {
                        right = v.x;
                    }

                    if v.y < top {
                        top = v.y;
                    }

                    if v.y > bottom {
                        bottom = v.y;
                    }
                }

                AABB::from_points(V2::new(left, top), V2::new(right, bottom))
            }
        }
    }

    /// The width and height of the box needed to fully contain the shape.
    pub fn extents(&self) -> V2 {
        self.aabb().extents
    }

    /// Scale the shape in x and y.
    pub fn into_scaled(self, scale: &V2) -> Shape {
        match self {
            Shape::Box { upper, lower } => Shape::Box {
                upper: upper * *scale,
                lower: lower * *scale,
            },
            Shape::Polygon { vertices } => {
                let vertices: Vec<V2> = vertices.into_iter().map(|v| v * *scale).collect();
                Shape::Polygon { vertices }
            }
        }
    }

    /// Return a new shape translated by a vector
    pub fn translated(&self, v: &V2) -> Shape {
        match &self {
            Shape::Box { upper, lower } => Shape::Box {
                upper: *upper + *v,
                lower: *lower + *v,
            },
            Shape::Polygon { vertices } => Shape::Polygon {
                vertices: vertices.iter().map(|p| *p + *v).collect(),
            },
        }
    }


    /// A list of all the vertices in this shape.
    pub fn vertices(&self) -> Vec<V2> {
        match self {
            Shape::Box { lower, upper } => vec![
                *lower,
                V2::new(upper.x, lower.y),
                *upper,
                V2::new(lower.x, upper.y),
            ],
            Shape::Polygon { vertices } => vertices.clone(),
        }
    }

    /// A list of all the vertices in this shape with the first vertex cloned and
    /// appended to the end, closing the polygon.
    pub fn vertices_closed(&self) -> Vec<V2> {
        let mut vertices = self.vertices();
        let first_vertex = vertices.first().cloned();

        if first_vertex.is_none() {
            // This is an empty polygon, return an empty list
            return vec![];
        }

        let first_vertex = first_vertex.unwrap();

        vertices.push(first_vertex);

        vertices
    }

    /// A list of potential separating axes as unit vectors.
    /// See https://www.metanetsoftware.com/2016/n-tutorial-a-collision-detection-and-response#section1
    pub fn potential_separating_axes(&self) -> Vec<V2> {
        let vertices = self.vertices_closed();
        let mut out_vertices = vec![];

        for i in 1..vertices.len() {
            let p1 = vertices[i - 1];
            let p2 = vertices[i];
            let v = p1 - p2;
            // get the perpendicular vector
            let pv = V2::new(-v.y, v.x);
            out_vertices.push(pv.unitize());
        }

        out_vertices.into_iter().filter_map(|v| v).collect()
    }


    /// Returns the midpoints of each edge in the shape.
    /// Used for (at least) debug drawing info.
    pub fn midpoints(&self) -> Vec<V2> {
        let vs = self.vertices_closed();

        (1..vs.len())
            .map(|i| {
                let p2 = vs[i];
                let p1 = vs[i - 1];
                p1 + (p2 - p1).scalar_mul(0.5)
            })
            .collect()
    }


    /// Returns a simple range of the vec.
    pub fn range(v: &[f32]) -> (f32, f32) {
        let start = (std::f32::INFINITY, std::f32::NEG_INFINITY);
        v.iter().fold(start, |(min, max), n| {
            (f32::min(min, *n), f32::max(max, *n))
        })
    }


    /// Return the ranged projection of this shape on an axis.
    pub fn ranged_projection_on(
        &self,
        p: V2,    // the world position of this shape
        axis: V2, // the axis we're projecting onto
    ) -> (f32, f32) {
        let points1d = self
            .vertices()
            .into_iter()
            .map(|v| {
                let loc = p + v;
                axis.dot(loc)
            })
            .collect::<Vec<_>>();
        Self::range(&points1d)
    }


    /// Returns the minimum translation vector needed to push an intersecting
    /// shape out of intersection. If the two are not intersecting it returns
    /// `None`. This should be called after the broadphase of detection.
    pub fn mtv_apart(
        &self,
        this_position: V2,   // This shape's world location
        other_shape: &Shape, // The other shape
        other_position: V2,  // The other shape's world location
    ) -> Option<V2> {
        // Maintain the smallest axis overlap that we'll later use as the mtv
        let mut overlap: Option<(f32, V2)> = None;

        let mut axes: Vec<V2> = self.potential_separating_axes();
        axes.extend(other_shape.potential_separating_axes());

        for axis in axes {
            let (my_start, my_end) = self.ranged_projection_on(this_position, axis);
            let (their_start, their_end) = other_shape.ranged_projection_on(other_position, axis);
            let does_collide = my_end > their_start && my_start < their_end;
            if !does_collide {
                // Early exit! These shapes don't overlap
                return None;
            }

            let this_overlap = my_end - their_start;
            let last_overlap = overlap.map(|(o, _)| o).unwrap_or(std::f32::INFINITY);
            if this_overlap.abs() < last_overlap.abs() {
                overlap = Some((this_overlap, axis));
            }
        }

        overlap.map(|(o, v)| v.scalar_mul(o))
    }
}


/// Lots of things have shapes in OG games and they change often.
/// We need to sync shapes with an rtree resource so shape's storage is a flagged
/// storage type.
/// @See https://slide-rs.github.io/specs/12_tracked.html
impl Component for Shape {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
