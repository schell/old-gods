use super::super::geom::V2;


#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LineSegment {
  pub a: V2,
  pub b: V2
}


impl LineSegment {
  pub fn new(a: V2, b:V2) -> LineSegment {
    LineSegment{a, b}
  }

  pub fn intersection_with(&self, l:LineSegment) -> Option<V2> {
    let r:V2 = self.b - self.a;
    let s = l.b - l.a;
    let rxs = r.cross(s);
    let qp = l.a - self.a;
    let qpxr = qp.cross(r);
    let rxs_is_zero = rxs.abs() < 1e-10;

    if rxs_is_zero {
      None
    } else {
      let t:f32 = qp.cross(s) / rxs;
      let u:f32 = qpxr / rxs;

      // If 0 <= t <= 1 and 0 <= u <= 1
      // the two line segments meet at the point p + t r = q + u s.
      if (0.0 <= t && t <= 1.0) && (0.0 <= u && u <= 1.0) {
        // We can calculate the intersection point using either t or u.
        Some(self.a + r.scalar_mul(t))
      } else {
        // Otherwise, the two line segments are not parallel but do not intersect.
        None
      }
    }
  }

  /// Return the vector difference (b - a).
  pub fn vector_difference(&self) -> V2 {
    self.b - self.a
  }
}
