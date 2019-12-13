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
use std::collections::HashMap;

use super::super::components::{Player, Exile, OriginOffset, Position};
use super::super::systems::screen::Screen;
use super::super::geom::V2;


#[derive(Debug, Clone, PartialEq)]
pub enum Trigger {
  /// This sound should always be playing when it is within range.
  /// This means the sound should loop indefinitely.
  Loop,

  /// This sound should only play once when it comes into range and is not exiled.
  Once
}


#[derive(Debug, Clone)]
/// A component for sound effects placed within the map.
pub struct Sound {
  /// The sound file for this sound
  pub file: String,

  /// The type of triggering this sound adheres to
  pub trigger: Trigger,

  /// The volume this sound should play at. This should be a number between 0.0
  /// and 1.0
  pub volume: f32,

  /// The channel this sound is currently playing on.
  pub channel: Option<Channel>
}


impl Sound {
  pub fn channel_volume(&self) -> i32 {
    (sdl2::mixer::MAX_VOLUME as f32 * self.volume) as i32
  }

  /// Release the sound, ending playback and freeing the channel.
  pub fn release(&mut self) {
    self
      .channel
      .map(|chan| {
        println!("Releasing the channel");
        chan
          .unregister_all_effects()
          .expect("Could not unregister channel effects");
        chan
          .pause();
        chan
          .halt();
        chan
          .expire(0);
      });
    self.channel = None;
  }
}


impl Component for Sound {
  type Storage = HashMapStorage<Self>;
}


#[derive(Debug, Clone)]
/// A component for music that plays everywhere.
pub struct Music(pub Sound);


impl Component for Music {
  type Storage = HashMapStorage<Self>;
}


pub struct SoundSystem {
  pub chunks: HashMap<String, Chunk>
}


impl SoundSystem {
  /// The channel group used for sound effects.
  fn fx_group(&self) -> Group {
    Group(-1)
  }

  fn next_fx_channel(&self) -> Channel {
    let channel =
      self
      .fx_group()
      .find_available();
    if let Some(channel) = channel {
      channel
    } else {
      // Increase the number of channels
      let num_channels =
        self
        .fx_group()
        .count();
      sdl2::mixer::allocate_channels(num_channels + 1);
      self
        .fx_group()
        .find_available()
        .expect("No sound fx channels are available")
    }
  }
}


impl SoundSystem {
  pub fn new() -> SoundSystem {
    //let mut timer = ctx.timer().unwrap();
    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size).unwrap();
    let _mixer_context =
      sdl2::mixer::init(InitFlag::OGG)
      .unwrap();

    //{
    //  let n = sdl2::mixer::get_chunk_decoders_number();
    //  println!("available chunk(sample) decoders: {}", n);
    //  for i in 0..n {
    //    println!("  decoder {} => {}", i, sdl2::mixer::get_chunk_decoder(i));
    //  }
    //}

    //{
    //  let n = sdl2::mixer::get_music_decoders_number();
    //  println!("available music decoders: {}", n);
    //  for i in 0..n {
    //    println!("  decoder {} => {}", i, sdl2::mixer::get_music_decoder(i));
    //  }
    //}

    //println!("query spec => {:?}", sdl2::mixer::query_spec());

    SoundSystem {
      chunks: HashMap::new()
    }
  }

  pub fn get_channel_and_play(&mut self, sound: &mut Sound) {
    println!("Creating a new channel");
    // Make sure this sound has a channel to play on
    let channel =
      self
      .next_fx_channel();
    // Load the sound file into our chunks cache if necessary
    let chunk = {
      if !self.chunks.contains_key(&sound.file) {
        let chunk =
          Chunk::from_file(sound.file.clone())
          .map_err(|e| format!(
            "Cannot load sound {:?} {}",
            sound.file,
            e
          ))
          .unwrap();
        self
          .chunks
          .insert(sound.file.clone(), chunk);
      }

      self
        .chunks
        .get(&sound.file)
        .unwrap()
    };

    let loops =
      if sound.trigger == Trigger::Loop {
        -1
      } else {
        0
      };
    channel
      .play(chunk, loops)
      .map_err(|e| format!("Cannot play sound {:?}", e))
      .unwrap();
    sound.channel =
      Some(channel);
  }
}


impl<'a> System<'a> for SoundSystem {
  type SystemData = (
    ReadStorage<'a, Player>,
    Entities<'a>,
    ReadStorage<'a, Exile>,
    WriteStorage<'a, Music>,
    ReadStorage<'a, OriginOffset>,
    ReadStorage<'a, Position>,
    Read<'a, Screen>,
    WriteStorage<'a, Sound>,
  );

  fn run(
    &mut self,
    (
      creatures,
      entities,
      exiles,
      mut musics,
      offsets,
      positions,
      screen,
      mut sounds
    ): Self::SystemData
  ) {
    // Find the greatest distance a player could see
    let max_distance =
      screen
      .get_size()
      .scalar_mul(0.3)
      .magnitude();
    // Find the zeroeth player
    let player_pos:V2 =
    (&entities, &creatures, &positions)
      .join()
      .filter_map(|(e, c, p)| {
        if c.0 == 0 {
          offsets
            .get(e)
            .map(|&OriginOffset(o)| {
              p.0 + o
            })
            .or(Some(p.0))
        } else {
          None
        }
      })
      .collect::<Vec<_>>()
      .first()
      .cloned()
      .unwrap_or(screen.get_focus());

    // Run through all the sounds that need to be triggered
    for (_ent, mut sound, &Position(p), ()) in (&entities, &mut sounds, &positions, !&exiles).join() {
      // TODO: Only check the sounds that are within a certain range of the
      // player position
      let (distance, angle, can_hear_sound) = {
        // Find the player's proximity to the sound
        let proximity =
          player_pos
          .distance_to(&p);
        // adjust for the max distance of seeing things and the volume
        // the volume effectively lowers the distance at which things can be
        // heard
        let percent =
          proximity / (max_distance * sound.volume);
        // Scale out of 255
        let distance =
          (255.0 * percent) as u8;

        // Get the angle as well
        let v =
          player_pos - p;
        let a =
          v.angle_degrees();
        let angle =
          (270 + a) % 360;

        (distance, angle, percent < 1.0)
      };

      if can_hear_sound {
        if sound.channel.is_none() {
          self
            .get_channel_and_play(&mut sound);
        }

        // set the channel effects
        sound
          .channel
          .map(|chan| {
            chan
              .set_position(angle, distance)
              .unwrap();
          });
      } else {
        // This sound cannot be heard, release the channel if it has one
        sound
          .release();
      }
    }

    // Run through all the musics
    for (_ent, music, &Position(p), ()) in (&entities, &mut musics, &positions, !&exiles).join() {
      let is_on_screen =
        screen
        .aabb()
        .contains_point(&p);
      if !is_on_screen {
        if music.0.channel.is_some() {
          music.0.release();
        }
      } else if music.0.channel.is_none() {
        self
          .get_channel_and_play(&mut music.0);
        // set the channel effects
        music
          .0
          .channel
          .map(|chan| {
            let volume =
              music.0.volume * 255.0;
            chan
              .set_volume(volume as i32);
          });
      }
    }
  }
}
