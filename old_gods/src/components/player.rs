use specs::prelude::{Component, Entities, Entity, HashMapStorage, Join, ReadStorage};


/// A component for designating the maximum velocity of an entity.
#[derive(Clone, Debug)]
pub struct MaxSpeed(pub f32);


impl MaxSpeed {
    pub fn tiled_key() -> String {
        "max_speed".to_string()
    }
}


impl Component for MaxSpeed {
    type Storage = HashMapStorage<MaxSpeed>;
}


#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
/// All the AIs in our game.
pub enum AI {
    /// An AI that just walks left.
    WalksLeft,
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
/// A player, controlled by an sdl controller.
pub struct Player(pub u32);


impl Player {
    pub fn tiled_key() -> String {
        "control".to_string()
    }

    pub fn get_entity<'a>(
        &self,
        entities: &Entities<'a>,
        players: &ReadStorage<'a, Player>,
    ) -> Option<Entity> {
        for (entity, player) in (entities, players).join() {
            if player == self {
                return Some(entity);
            }
        }
        None
    }
}


impl Component for Player {
    type Storage = HashMapStorage<Self>;
}


/// A component for suspending control of an entity without exiling it.
pub struct SuspendPlayer;


impl Component for SuspendPlayer {
    type Storage = HashMapStorage<SuspendPlayer>;
}
