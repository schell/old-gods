use old_gods::{
    prelude::{
        Component, Easing, HashMapStorage, OriginOffset, Position, Rendering, Shape, Tween,
        TweenParam, Velocity, AABB, V2,
    },
};
use specs::prelude::*;
use std::f32::consts::PI;


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
    pub items: Vec<Item>,

    /// A place to store the next angle to use for throwing an item out
    /// of the inventory.
    pub next_ejection_angle: u32,
}


impl Inventory {
    pub fn new(items: Vec<Item>) -> Inventory {
        Inventory {
            items,
            next_ejection_angle: 0,
        }
    }

    pub fn remove_item(&mut self, item: &Item) -> Result<(), String> {
        let mut may_ndx = None;
        for (item_here, ndx) in self.items.iter().zip(0..) {
            if item_here == item {
                may_ndx = Some(ndx);
                break;
            }
        }
        let ndx = may_ndx.ok_or("Could not find item")?;
        self.items.remove(ndx);
        Ok(())
    }

    /// Dequeue the next item ejection angle. This is nice for
    /// a good item dropping effect.
    fn dequeue_ejection_in_radians(&mut self) -> f32 {
        let n = self.next_ejection_angle as usize;
        self.next_ejection_angle += 1;

        ITEM_PLACEMENTS[n % ITEM_PLACEMENTS.len()]
    }

    /// Add the item, stacking it in an available stack if possible.
    pub fn add_item(&mut self, item: Item) {
        for existing_item in self.items.iter_mut() {
            if existing_item.stack.is_some() && existing_item.name == item.name {
                let stack = existing_item.stack.as_mut().unwrap();
                *stack += item.stack.unwrap_or(1);
                return;
            }
        }

        self.items.push(item);
    }

    /// Finds a position around the holder that's out of the way, and then throws the item there.
    pub fn throw_item_with_index_onto_the_map(
        &mut self,
        ndx: usize,
        starting_loc: V2,
        from_aabb: AABB,
        entities: &Entities,
        lazy: &LazyUpdate,
    ) {
        let item = self.items.remove(ndx);
        let item_aabb = item.shape.aabb();
        // From there we must offset it some amount to account for
        // the barriers of each
        let radius = {
            let f = from_aabb.greater_extent();
            let i = item_aabb.greater_extent();
            f32::max(f, i)
        };

        // Place the item
        let radians = self.dequeue_ejection_in_radians();
        let dv = V2::new(f32::cos(radians), f32::sin(radians));
        let loc = starting_loc + (dv.scalar_mul(radius));

        // Fuckit! Throw the item!
        let speed = 100.0;
        let starting_v = dv.scalar_mul(speed);
        let subject = lazy
            .create_entity(entities)
            .with(Position(loc))
            .with(Velocity(starting_v))
            .with(item.rendering.clone())
            .with(item.shape.clone());
        let subject = if let Some(offset) = item.offset {
            subject.with(offset)
        } else {
            subject
        };
        let subject = subject.with(item).build();
        // Tween the item flying out of the inventory, eventually stopping.
        let _ = lazy
            .create_entity(entities)
            .with(Tween::new(
                subject,
                TweenParam::Velocity(starting_v, V2::origin()),
                Easing::Linear,
                0.5,
            ))
            .build();

        println!("dv:{:?} radius:{:?} vel:{:?}", dv, radius, starting_v);
    }
}


impl Component for Inventory {
    type Storage = HashMapStorage<Inventory>;
}
