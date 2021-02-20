mod button_map;

use button_map::ButtonMap;
use std::error::Error;

use crate::synth::{Synth, Voice};

pub struct DefaultSynth {
    button_map: ButtonMap,
    sample_rate: f32,

    target_master_volume: f32,
    master_volume: f32,
    key_velocity: bool,
}

impl DefaultSynth {
    pub fn new(config_file: &str, sample_rate: f32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            button_map: ButtonMap::from_toml(config_file)?,
            sample_rate,
            target_master_volume: 1.0,
            master_volume: 1.0,
            key_velocity: true,
        })
    }
}

impl Synth for DefaultSynth {
    type Voice = DefaultVoice;

    fn param_change(&mut self, param: u8, value: f32) {
        if param == self.button_map.master_volume {
            self.target_master_volume = value;
        }

        if param == self.button_map.key_velocity {
            self.key_velocity = value > 0.5;
        }
    }

    fn notify_buffer(&mut self) {}

    fn step_frame(&mut self) {
        self.master_volume = 0.95 * self.master_volume + 0.05 * self.target_master_volume;
    }
}


pub struct DefaultVoice {
    pitch: f32,
    vel: f32,
    released: bool,

    t: f32,
    decay_mult: f32,
}

impl Voice<DefaultSynth> for DefaultVoice {
    fn new(pitch: f32, vel: f32, synth: &DefaultSynth) -> Self {
        Self {
            pitch,
            vel: if synth.key_velocity { vel } else { 1.0 },
            released: false,
            t: 0.0,
            decay_mult: 1.0,
        }
    }

    fn step_frame(&mut self, synth: &DefaultSynth) -> f32 {
        if self.released {
            self.decay_mult *= 0.999;
        }

        let volume = self.vel * synth.master_volume;

        let val = self.vel * (self.t * 2.0 * std::f32::consts::PI).sin();
        self.t += self.pitch / synth.sample_rate;
        self.t %= 1.0;
        val * self.decay_mult * volume
    }

    fn notify_release(&mut self) {
        self.released = true;
    }

    fn is_done(&self) -> bool {
        self.decay_mult < 0.0001
    }
}
