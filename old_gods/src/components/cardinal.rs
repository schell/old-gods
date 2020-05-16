//! Data for cardinal directions.
use specs::prelude::{Component, HashMapStorage};

use super::V2;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cardinal {
    North,
    East,
    South,
    West,
}


impl Cardinal {
    pub fn try_from_str(s: &str) -> Option<Cardinal> {
        match s {
            "north" => Some(Cardinal::North),
            "east" => Some(Cardinal::East),
            "south" => Some(Cardinal::South),
            "west" => Some(Cardinal::West),
            _ => None,
        }
    }

    //pub fn from_keycode(keycode: &Keycode) -> Option<Cardinal> {
    //  match keycode {
    //    Keycode::J => {Some(Cardinal::South)}
    //    Keycode::K => {Some(Cardinal::North)}
    //    Keycode::H => {Some(Cardinal::West)}
    //    Keycode::L => {Some(Cardinal::East)}
    //    _ => None
    //  }
    //}

    /// Returns a cardinal direction from the vector, if possible.
    /// Returns None if the x and y components of the vector are equal.
    pub fn from_v2(v: &V2) -> Option<Cardinal> {
        if v.x.abs() > v.y.abs() {
            // East or West
            if v.x < 0.0 {
                Some(Cardinal::West)
            } else {
                Some(Cardinal::East)
            }
        } else if v.x.abs() < v.y.abs() {
            // North or South
            if v.y < 0.0 {
                Some(Cardinal::North)
            } else {
                Some(Cardinal::South)
            }
        } else {
            None
        }
    }


    /// Returns the caradinal expressed as a vector.
    pub fn as_v2(&self) -> V2 {
        match self {
            Cardinal::North => V2::new(0.0, -1.0),
            Cardinal::East => V2::new(1.0, 0.0),
            Cardinal::South => V2::new(0.0, 1.0),
            Cardinal::West => V2::new(-1.0, 0.0),
        }
    }


    pub fn opposite(&self) -> Cardinal {
        match self {
            Cardinal::East => Cardinal::West,
            Cardinal::West => Cardinal::East,
            Cardinal::North => Cardinal::South,
            Cardinal::South => Cardinal::North,
        }
    }
}


impl Component for Cardinal {
    type Storage = HashMapStorage<Self>;
}
