use spade::BoundingRect;
//use sdl2::rect::Rect;

use super::v2::V2;
use super::super::prelude::Shape;


#[derive(Debug, Clone, PartialEq, Copy)]
/// An axis aligned bounding box.
pub struct AABB {
  /// The top left of the box.
  pub top_left: V2,

  /// The width and height of the box.
  pub extents: V2
}


impl AABB {
  pub fn new(x:f32, y:f32, w:f32, h:f32) -> AABB {
    AABB {
      top_left: V2::new(x,y),
      extents: V2::new(w, h)
    }
  }


  pub fn from_points(lower: V2, upper: V2) -> AABB {
    AABB {
      top_left: lower,
      extents: upper - lower
    }
  }


  // Returns a new AABB with no width or height,
  // positioned at the origin.
  pub fn identity() -> AABB {
    AABB {
      top_left: V2::origin(),
      extents: V2::origin()
    }
  }

  pub fn translate(&self, t: &V2) -> AABB {
    let mut aabb = self.clone();
    aabb.top_left = aabb.top_left + *t;
    aabb
  }

  /// The left x coord.
  pub fn left(&self) -> f32 {
    self.top_left.x
  }

  /// The right x coord.
  pub fn right(&self) -> f32 {
    self.top_left.x + self.extents.x
  }

  /// The top y coord.
  pub fn top(&self) -> f32 {
    self.top_left.y
  }

  /// The bottom y coord.
  pub fn bottom(&self) -> f32 {
    self.top_left.y + self.extents.y
  }

  /// The center of the aabb.
  pub fn center(&self) -> V2 {
    self.top_left + self.extents.scalar_mul(0.5)
  }

  pub fn lower(&self) -> V2 {
    self.top_left
  }

  pub fn upper(&self) -> V2 {
    self.top_left + self.extents
  }

  /// The half-width vector.
  pub fn hwv(&self) -> V2 {
    V2::new(self.extents.x / 2.0, 0.0)
  }

  /// The half-height vector.
  pub fn hhv(&self) -> V2 {
    V2::new(0.0, self.extents.y / 2.0)
  }

  /// The greater of the two extents
  pub fn greater_extent(&self) -> f32 {
    f32::max(self.extents.x, self.extents.y)
  }

  /// The width
  pub fn width(&self) -> f32 {
    self.extents.x
  }

  /// The height
  pub fn height(&self) -> f32 {
    self.extents.y
  }

  /// Whether or not the aabb contans the point.
  pub fn contains_point(&self, p: &V2) -> bool {
    p.x >= self.left()
      && p.x <= self.right()
      && p.y >= self.top()
      && p.y <= self.bottom()
  }

  pub fn collides_on_x(&self, aabb: &AABB) -> bool {
    self.right() > aabb.left()
      && self.left() < aabb.right()
  }

  pub fn collides_on_y(&self, aabb: &AABB) -> bool {
    self.bottom() > aabb.top()
      && self.top() < aabb.bottom()
  }

  /// Whether or not the aabb collides with another.
  pub fn collides_with(&self, aabb:&AABB) -> bool {
    // Does it intersect
    self.collides_on_x(aabb)
      && self.collides_on_y(aabb)
  }

  /// Return the minimum translation vector, if the two are intersecting.
  /// Returns the mtv needed to push the given AABB out of intersection.
  pub fn mtv_apart(&self, aabb:&AABB) -> Option<V2> {
    if self.collides_with(aabb) {
      // The two are intersecting.
      // Figure out the minimum tranlation that will push them out of
      // intersection.
      let dx = if self.right() < aabb.right() {
        self.right() - aabb.left()
      } else {
        self.left() - aabb.right()
      };
      let dy = if self.bottom() < aabb.bottom() {
        self.bottom() - aabb.top()
      } else {
        -(aabb.bottom() - self.top())
      };
      let dxy = if dx.abs() < dy.abs() {
        V2::new(dx, 0.0)
      } else {
        V2::new(0.0, dy)
      };
      Some(dxy)
    } else {
      None
    }
  }

  /// Return the minimum translation vector needed to allow an AABB to fit within
  /// this AABB.
  /// Returns None if the given AABB cannot fit.
  pub fn mtv_inside(&self, aabb: &AABB) -> Option<V2> {
    let cannot_fit =
      aabb.width() > self.width()
      || aabb.height() > self.height();
    if cannot_fit {
      return None;
    }

    let dy = {
      // Move the y by aligning the tops or bottoms
      let top_delta =
        self.top() - aabb.top();
      let bottom_delta =
        self.bottom() - aabb.bottom();

      if top_delta.abs() > bottom_delta.abs() {
        top_delta
      } else {
        bottom_delta
      }
    };

    let dx = {
      // move the x by aligning the lefts or rights
      let left_delta =
        self.left() - aabb.left();
      let right_delta =
        self.right() - aabb.right();

      if left_delta.abs() > right_delta.abs() {
        left_delta
      } else {
        right_delta
      }
    };

    Some(V2::new(dx, dy))
  }

  /// Returns the minimum translation vector needed to push
  /// the given AABB so the two are touching.
  ///
  /// ```
  /// extern crate engine;
  /// use engine::geom::{V2, AABB};
  ///
  /// let a = AABB {
  ///   top_left: V2::new(0.0, 0.0),
  ///   extents: V2::new(10.0, 10.0)
  /// };
  /// let b = AABB {
  ///   top_left: V2::new(20.0, 20.0),
  ///   extents: V2::new(10.0, 10.0)
  /// };
  /// assert_eq!(a.mtv_contact(&b), V2::new(-10.0, -10.0));
  ///
  /// let c = AABB {
  ///   top_left: V2::new(20.0, -20.0),
  ///   extents: V2::new(10.0, 10.0)
  /// };
  /// assert_eq!(a.mtv_contact(&c), V2::new(-10.0, 10.0));
  ///
  /// let d = AABB {
  ///   top_left: V2::new(-20.0, -20.0),
  ///   extents: V2::new(10.0, 10.0)
  /// };
  /// assert_eq!(a.mtv_contact(&d), V2::new(10.0, 10.0));
  ///
  /// let e = AABB {
  ///   top_left: V2::new(10.0, 0.0),
  ///   extents: V2::new(10.0, 10.0)
  /// };
  /// assert_eq!(a.mtv_contact(&e), V2::new(0.0, 0.0));
  ///
  /// let point_aabb = AABB {
  ///   top_left: V2::new(1.0, 0.0),
  ///   extents: V2::origin()
  /// };
  /// assert_eq!(a.mtv_contact(&point_aabb), V2::new(0.0, 0.0));
  /// ```
  pub fn mtv_contact(&self, aabb:&AABB) -> V2 {
    let dx = if self.collides_on_x(aabb) {
      if self.top() == aabb.bottom() {
        return V2::new(0.0, 0.0);
      }
      if self.bottom() == aabb.top() {
        return V2::new(0.0, 0.0);
      }
      0.0
    } else {
      if self.right() < aabb.left() {
        -(aabb.left() - self.right())
      } else {
        self.left() - aabb.right()
      }
    };

    let dy = if self.collides_on_y(aabb) {
      if self.right() == aabb.left() {
        return V2::new(0.0, 0.0);
      }
      if self.left() == aabb.right() {
        return V2::new(0.0, 0.0);
      }
      0.0
    } else {
      if self.bottom() < aabb.top() {
        -(aabb.top() - self.bottom())
      } else {
        self.top() - aabb.bottom()
      }
    };
    V2::new(dx, dy)
  }


  pub fn to_mbr(&self) -> BoundingRect<V2> {
    BoundingRect::from_points(vec![
      self.top_left,
      self.top_left + self.extents
    ])
  }


  pub fn from_mbr(bounds: &BoundingRect<V2>) -> AABB {
    AABB {
      top_left: bounds.lower(),
      extents: bounds.upper() - bounds.lower(),
    }
  }


  //pub fn to_rect(&self) -> Rect {
  //  Rect::new(
  //    f32::round(self.top_left.x) as i32,
  //    f32::round(self.top_left.y) as i32,
  //    f32::round(self.extents.x) as u32,
  //    f32::round(self.extents.y) as u32
  //  )
  //}

  pub fn round(&self) -> AABB {
    let mut aabb = self.clone();
    aabb.top_left.x = aabb.top_left.x.round();
    aabb.top_left.y = aabb.top_left.y.round();
    aabb.extents.x = aabb.extents.x.round();
    aabb.extents.y = aabb.extents.y.round();
    aabb
  }

  pub fn to_shape(&self) -> Shape {
    Shape::Box {
      lower: self.top_left,
      upper: self.upper()
    }
  }


  pub fn set_center(&mut self, center: &V2) {
    self.top_left.x = center.x - self.extents.x / 2.0;
    self.top_left.y = center.y - self.extents.y / 2.0;
  }


  pub fn set_lower(&mut self, lower: &V2) {
    self.extents = *lower - self.top_left;
  }


  pub fn scale_needed_to_fit_inside(
    inside: V2,
    outside: V2
  ) -> f32 {
    let width_scale = outside.x / inside.x;
    let height_scale = outside.y / inside.y;
    f32::min(width_scale, height_scale)
  }


  pub fn aabb_to_aspect_fit_inside(
    inside: V2,
    outside: V2
  ) -> AABB {
    let scale =
      Self::scale_needed_to_fit_inside(inside, outside);
    let extents =
      V2::new(scale * inside.x, scale * inside.y);
    let top_left =
      V2::new((outside.x - extents.x) / 2.0, (outside.y - extents.y) / 2.0);
    AABB {
      top_left,
      extents
    }
  }


  /// Combine two AABBs, forming an AABB that contains them both.
  pub fn union(a: &AABB, b: &AABB) -> AABB {
    let upper =
      V2::new(
        f32::min(a.left(), b.left()),
        f32::min(a.top(), b.top())
      );
    let lower =
      V2::new(
        f32::max(a.right(), b.right()),
        f32::max(a.bottom(), b.bottom())
      );
    AABB::from_points(upper, lower)
  }
}
