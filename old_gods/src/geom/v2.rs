use spade::PointN;
//use sdl2::rect::Point;
use std::{fmt::Debug, ops::*};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}


impl Sub for V2 {
    type Output = V2;
    fn sub(self, other: V2) -> V2 {
        V2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}


impl Add for V2 {
    type Output = V2;
    fn add(self, other: V2) -> V2 {
        V2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}


impl AddAssign for V2 {
    fn add_assign(&mut self, other: V2) {
        self.x += other.x;
        self.y += other.y;
    }
}


impl SubAssign for V2 {
    fn sub_assign(&mut self, other: V2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}


impl Mul for V2 {
    type Output = V2;
    fn mul(self, v: V2) -> V2 {
        V2 {
            x: self.x * v.x,
            y: self.y * v.y,
        }
    }
}


impl Div for V2 {
    type Output = V2;
    fn div(self, v: V2) -> V2 {
        V2 {
            x: self.x / v.x,
            y: self.y / v.y,
        }
    }
}


impl V2 {
    pub fn new(x: f32, y: f32) -> V2 {
        V2 { x, y }
    }

    pub fn origin() -> V2 {
        V2::new(0.0, 0.0)
    }

    pub fn normal(&self) -> V2 {
        V2 {
            x: self.y * (-1.0),
            y: self.x,
        }
    }

    pub fn unitize(&self) -> Option<V2> {
        let m = self.magnitude();
        if m == 0.0 {
            None
        } else {
            Some(V2 {
                x: self.x / m,
                y: self.y / m,
            })
        }
    }

    pub fn dot(&self, other: V2) -> f32 {
        (self.x * other.x) + (self.y * other.y)
    }

    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn distance_to(&self, other: &V2) -> f32 {
        (*other - *self).magnitude()
    }

    /// Normally the cross product gives you a vector
    /// orthogonal to the two param vectors, but since
    /// this is all in 2d, you only get the z component,
    /// hence this function returns an f32.
    pub fn cross(&self, v: V2) -> f32 {
        self.x * v.y - self.y * v.x
    }

    pub fn scalar_mul(&self, n: f32) -> V2 {
        V2 {
            x: self.x * n,
            y: self.y * n,
        }
    }

    pub fn translate(&self, v: &V2) -> V2 {
        *self + *v
    }

    pub fn angle_radians(&self) -> f32 {
        f32::atan2(self.y, self.x)
    }

    pub fn angle_degrees(&self) -> i16 {
        let radians = self.angle_radians();
        (radians * 57.29578) as i16
    }

    //pub fn into_point(self) -> Point {
    //  Point::new(
    //    f32::round(self.x) as i32,
    //    f32::round(self.y) as i32,
    //  )
    //}
}


impl PointN for V2 {
    type Scalar = f32;

    fn dimensions() -> usize {
        2
    }

    fn from_value(value: Self::Scalar) -> Self {
        V2::new(value, value)
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.x,
            _ => &self.y,
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct KeyVal<K, V> {
    pub key: K,
    pub value: Option<V>,
}


impl<K: PointN, V: Clone + Debug + PartialEq> PointN for KeyVal<K, V> {
    type Scalar = K::Scalar;

    fn dimensions() -> usize {
        K::dimensions()
    }

    fn from_value(value: Self::Scalar) -> Self {
        KeyVal {
            key: K::from_value(value),
            value: None,
        }
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        self.key.nth(index)
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        self.key.nth_mut(index)
    }
}
