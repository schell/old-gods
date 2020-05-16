/// The SoundSystem handles playing music and sound.
use specs::prelude::*;
//use sdl2::mixer::{
//  AUDIO_S16LSB,
//  DEFAULT_CHANNELS,
//  Chunk,
//  Channel,
//  Group
//};
//use sdl2::mixer::InitFlag;

use super::super::prelude::{
    Exile, OriginOffset, Player, Position, Screen, SoundBlaster, JSON, V2,
};


#[derive(Debug, Clone)]
/// A component for sound effects placed within the map.
pub struct Sound {
    /// The sound file for this sound
    pub file: String,

    /// The volume this sound should play at. This should be a number between 0.0
    /// and 1.0
    pub volume: f32,

    /// Whether or not this sound autoplays when loaded.
    pub autoplay: bool,

    /// Whether or not this sound is "on the map".
    /// Being "on the map" means that it will be positioned spatially around the
    /// player and panned across the speakers.
    pub on_map: bool,
}


impl Component for Sound {
    type Storage = HashMapStorage<Self>;
}


pub struct SoundSystem {
    _blaster: SoundBlaster,
}


impl SoundSystem {
    // /// The channel group used for sound effects.
    // fn fx_group(&self) -> Group {
    //     Group(-1)
    // }

    // fn next_fx_channel(&self) -> Channel {
    //     let channel = self.fx_group().find_available();
    //     if let Some(channel) = channel {
    //         channel
    //     } else {
    //         // Increase the number of channels
    //         let num_channels = self.fx_group().count();
    //         sdl2::mixer::allocate_channels(num_channels + 1);
    //         self.fx_group()
    //             .find_available()
    //             .expect("No sound fx channels are available")
    //     }
    // }
}


impl Default for SoundSystem {
    fn default() -> SoundSystem {
        SoundSystem {
            _blaster: SoundBlaster::new(),
        }
    }
}


#[derive(SystemData)]
pub struct SoundSystemData<'a> {
    entities: Entities<'a>,
    _jsons: WriteStorage<'a, JSON>,
    players: ReadStorage<'a, Player>,
    exiles: ReadStorage<'a, Exile>,
    offsets: ReadStorage<'a, OriginOffset>,
    positions: ReadStorage<'a, Position>,
    screen: Read<'a, Screen>,
    sounds: WriteStorage<'a, Sound>,
}


impl<'a> System<'a> for SoundSystem {
    type SystemData = SoundSystemData<'a>;

    fn run(&mut self, mut data: SoundSystemData) {
        // Find the greatest distance a player could see
        let max_distance = data.screen.get_size().scalar_mul(0.3).magnitude();
        // Find the zeroeth player
        let player_pos: V2 = (&data.entities, &data.players, &data.positions)
            .join()
            .filter_map(|(e, c, p)| {
                if c.0 == 0 {
                    data.offsets
                        .get(e)
                        .map(|&OriginOffset(o)| p.0 + o)
                        .or(Some(p.0))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .first()
            .cloned()
            .unwrap_or_else(|| data.screen.get_focus());

        // Run through all the sounds that need to be triggered
        for (_ent, sound, &Position(p), ()) in (
            &data.entities,
            &mut data.sounds,
            &data.positions,
            !&data.exiles,
        )
            .join()
        {
            // TODO: Only check the sounds that are within a certain range of the
            // player position
            let (_distance, _angle, _can_hear_sound) = {
                // Find the player's proximity to the sound
                let proximity = player_pos.distance_to(&p);
                // adjust for the max distance of seeing things and the volume
                // the volume effectively lowers the distance at which things can be
                // heard
                let percent = proximity / (max_distance * sound.volume);
                // Scale out of 255
                let distance = (255.0 * percent) as u8;

                // Get the angle as well
                let v = player_pos - p;
                let a = v.angle_degrees();
                let angle = (270 + a) % 360;

                (distance, angle, percent < 1.0)
            };

            //if can_hear_sound {
            //} else {
            //}
        }
    }
}
