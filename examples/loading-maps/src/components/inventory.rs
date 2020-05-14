use super::super::systems::looting::Loot;
use old_gods::prelude::{Component, HashMapStorage, OriginOffset, Rendering, Shape};
use std::{f32::consts::PI, slice::Iter};


/// An entity with an item component can be kept in an inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// The name of this item.
    pub name: String,

    /// Whether or not this item is usable by itself.
    pub usable: bool,

    /// If this item can be stacked the Option type holds
    /// the count of the stack.
    pub stack: Option<usize>,

    /// How to render this item.
    pub rendering: Rendering,

    /// The shape of the item.
    pub shape: Shape,

    /// An origin, if it exists.
    pub offset: Option<OriginOffset>,

    /// Is this a barrier?
    pub is_barrier: bool,
}


impl Component for Item {
    type Storage = HashMapStorage<Item>;
}


const ITEM_PLACEMENTS: [f32; 16] = [
    0.0,
    PI / 2.0,
    PI,
    3.0 * PI / 2.0,
    PI / 4.0,
    3.0 * PI / 4.0,
    5.0 * PI / 4.0,
    7.0 * PI / 4.0,
    PI / 6.0,
    PI / 3.0,
    2.0 * PI / 3.0,
    5.0 * PI / 6.0,
    7.0 * PI / 6.0,
    4.0 * PI / 3.0,
    5.0 * PI / 3.0,
    11.0 * PI / 6.0,
];

/// # Inventory
/// An inventory is a container of items.
#[derive(Debug, Clone)]
pub struct Inventory {
    /// The items that are inside this inventory.
    /// The inventory is a grid of items.
    items: Vec<Item>,

    /// A place to store the next angle to use for throwing an item out
    /// of the inventory.
    next_ejection_angle: u32,
}


impl Inventory {
    pub fn new(items: Vec<Item>) -> Inventory {
        Inventory {
            items,
            next_ejection_angle: 0,
        }
    }

    /// Dequeue the next item ejection angle. This is nice for
    /// a good item dropping effect.
    pub fn dequeue_ejection_in_radians(&mut self) -> f32 {
        let n = self.next_ejection_angle as usize;
        self.next_ejection_angle += 1;

        ITEM_PLACEMENTS[n % ITEM_PLACEMENTS.len()]
    }

    /// Add the item, stacking it in an available stack if possible.
    pub fn add_item(&mut self, item: Item) {
        if item.stack.is_some() {
            for prev_item in self.items.iter_mut() {
                if prev_item.stack.is_some() && prev_item.name == item.name {
                    let stack = prev_item.stack.as_mut().unwrap();
                    *stack += item.stack.unwrap_or(1);
                    return;
                }
            }
        } else {
            self.items.push(item);
        }
    }

    /// An iterator over the items.
    pub fn item_at_xy(&self, x: i32, y: i32) -> Option<&Item> {
        self.items.get(y as usize * Loot::COLS + x as usize)
    }

    /// Remove the item at the given index.
    pub fn remove(&mut self, ndx: usize) -> Option<Item> {
        if ndx < self.items.len() {
            Some(self.items.remove(ndx))
        } else {
            None
        }
    }

    /// Remove the item at the given x and y
    pub fn remove_xy(&mut self, x: usize, y: usize) -> Option<Item> {
        let ndx = y * Loot::COLS + x;
        self.remove(ndx)
    }

    /// The number of items in the inventory.
    pub fn item_len(&self) -> usize {
        self.items.len()
    }

    /// Replace all the items in the inventory.
    /// Returns the old items.
    pub fn replace_items(&mut self, items: Vec<Item>) -> Vec<Item> {
        std::mem::replace(&mut self.items, items)
    }

    /// An iterator over all items.
    pub fn item_iter(&self) -> Iter<Item> {
        self.items.iter()
    }
}


impl Component for Inventory {
    type Storage = HashMapStorage<Inventory>;
}
