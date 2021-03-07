mod button_map;
mod compressor;
mod low_pass;
mod rng;

use crate::util::*;
use anyhow::Result;
use button_map::ButtonMap;
use rng::Xoroshiro;
use std::error::Error;

use crate::synth::{Synth, Voice};

const HEADROOM: f32 = 0.25;

pub struct DefaultSynth {
    button_map: ButtonMap,
    sample_rate: f32,
    rng_state: rng::Xoroshiro,

    key_velocity: bool,

    target_master_volume: f32,
    master_volume: f32,

    target_attack_time: f32,
    target_decay_time: f32,
    target_sustain: f32,
    target_release_time: f32,
    attack_time: f32,
    decay_time: f32,
    sustain: f32,
    release_time: f32,

    osc1_waveform: f32,
    osc2_waveform: f32,
    target_osc_balance: f32,
    osc_balance: f32,

    target_filter_cutoff: f32,
    target_filter_resonance: f32,
    filter_cutoff: f32,
    filter_resonance: f32,

    target_distortion_pregain: f32,
    target_distortion_level: f32,
    target_distortion_mix: f32,
    distortion_pregain: f32,
    distortion_level: f32,
    distortion_mix: f32,

    filter_relative: bool,
    enable_compressor: bool,
}

impl DefaultSynth {
    pub fn new(config_file: &str, sample_rate: f32) -> Result<Self> {
        Ok(Self {
            button_map: ButtonMap::from_toml(config_file)?,
            sample_rate,
            rng_state: Xoroshiro::new(42),

            key_velocity: true,

            target_master_volume: 1.0,
            master_volume: 1.0,

            target_attack_time: 0.01,
            attack_time: 0.01,

            target_decay_time: 0.01,
            decay_time: 0.01,

            target_sustain: 1.0,
            sustain: 1.0,

            target_release_time: 0.01,
            release_time: 0.01,

            osc1_waveform: 0.0,
            osc2_waveform: 0.0,
            target_osc_balance: 0.5,
            osc_balance: 0.5,

            target_filter_cutoff: 1.0,
            filter_cutoff: 1.0,

            target_filter_resonance: 0.5,
            filter_resonance: 0.5,

            target_distortion_pregain: 0.0,
            target_distortion_level: 0.0,
            target_distortion_mix: 0.0,
            distortion_pregain: 0.0,
            distortion_level: 0.0,
            distortion_mix: 0.0,

            enable_compressor: false,
            filter_relative: false,
        })
    }
}

impl Synth for DefaultSynth {
    type Voice = DefaultVoice;

    fn param_change(&mut self, param: u8, value: f32) {
        if param == self.button_map.master_volume {
            self.target_master_volume = value;
        } else if param == self.button_map.key_velocity {
            self.key_velocity = value > 0.5;
        } else if param == self.button_map.volume_attack {
            self.target_attack_time = value.mixexp(0.01, 5.0);
        } else if param == self.button_map.volume_decay {
            self.target_decay_time = value.mixexp(0.01, 5.0);
        } else if param == self.button_map.volume_sustain {
            self.target_sustain = value;
        } else if param == self.button_map.volume_release {
            self.target_release_time = value.mixexp(0.01, 5.0);
        } else if param == self.button_map.osc1_waveform {
            self.osc1_waveform = value;
        } else if param == self.button_map.osc2_waveform {
            self.osc2_waveform = value;
        } else if param == self.button_map.osc_balance {
            self.target_osc_balance = value;
        } else if param == self.button_map.filter_cutoff {
            self.target_filter_cutoff = value;
        } else if param == self.button_map.filter_resonance {
            self.target_filter_resonance = value;
        } else if param == self.button_map.enable_compressor {
            self.enable_compressor = value > 0.5;
        } else if param == self.button_map.filter_relative {
            self.filter_relative = value > 0.5;
        } else if param == self.button_map.distortion_pregain {
            self.target_distortion_pregain = value;
        } else if param == self.button_map.distortion_level {
            self.target_distortion_level = value;
        } else if param == self.button_map.distortion_mix {
            self.target_distortion_mix = value;
        }
    }

    fn notify_buffer(&mut self) {}

    fn step_frame(&mut self) {
        self.attack_time = 0.95 * self.attack_time + 0.05 * self.target_attack_time;
        self.decay_time = 0.95 * self.decay_time + 0.05 * self.target_decay_time;
        self.sustain = 0.95 * self.sustain + 0.05 * self.target_sustain;
        self.release_time = 0.95 * self.release_time + 0.05 * self.target_release_time;
        self.master_volume = 0.95 * self.master_volume + 0.05 * self.target_master_volume;
        self.osc_balance = 0.95 * self.osc_balance + 0.05 * self.target_osc_balance;
        self.filter_cutoff = 0.95 * self.filter_cutoff + 0.05 * self.target_filter_cutoff;
        self.filter_resonance = 0.95 * self.filter_resonance + 0.05 * self.target_filter_resonance;
        self.distortion_pregain =
            0.95 * self.distortion_pregain + 0.05 * self.target_distortion_pregain;
        self.distortion_level = 0.95 * self.distortion_level + 0.05 * self.target_distortion_level;
        self.distortion_mix = 0.95 * self.distortion_mix + 0.05 * self.target_distortion_mix;
    }
}

pub struct DefaultVoice {
    pitch: f32,
    vel: f32,
    released: bool,

    t: f32,
    wave_t: f32,
    pre_release_volume: f32,
    release_time: f32,

    low_pass: low_pass::MystramFilter,
    compressor: compressor::Compressor,

    rng_state: Xoroshiro,
}

impl Voice<DefaultSynth> for DefaultVoice {
    fn new(pitch: f32, vel: f32, synth: &mut DefaultSynth) -> Self {
        let compressor = compressor::Compressor::new(synth.sample_rate, 50.0);

        Self {
            pitch,
            vel: if synth.key_velocity { vel } else { 1.0 },
            released: false,
            t: 0.0,
            wave_t: 0.0,
            release_time: 0.0,
            pre_release_volume: 0.0,

            low_pass: low_pass::MystramFilter::new(synth.sample_rate as f64),
            compressor,

            rng_state: Xoroshiro::new(synth.rng_state.next()),
        }
    }

    fn step_frame(&mut self, synth: &DefaultSynth) -> (f32, f32) {
        let adsr;

        if self.released {
            let dt = self.t - self.release_time;
            let release_perc = (dt / synth.release_time).clamp(0.0, 1.0);
            adsr = (1.0 - release_perc).powi(2) * self.pre_release_volume;
        } else {
            let attack_perc = (self.t / synth.attack_time).clamp(0.0, 1.0);
            adsr = 1.0 - (1.0 - attack_perc).powi(2);
            self.pre_release_volume = adsr;
        }

        let volume = self.vel * synth.master_volume * adsr * HEADROOM;

        let osc1 = if synth.osc1_waveform < 0.25 {
            // Sine wave.
            (self.wave_t * 2.0 * std::f32::consts::PI).sin()
        } else if synth.osc1_waveform < 0.5 {
            // Sawtooth.
            if self.wave_t < 0.5 {
                2.0 * self.wave_t
            } else {
                2.0 * (self.wave_t - 0.5) - 1.0
            }
        } else if synth.osc1_waveform < 0.75 {
            // Square.
            if self.wave_t < 0.5 {
                1.0
            } else {
                -1.0
            }
        } else {
            // Noise.
            2.0 * self.rng_state.next_float() - 1.0
        };

        let osc2 = if synth.osc2_waveform < 0.25 {
            // Sine wave.
            (self.wave_t * 2.0 * std::f32::consts::PI).sin()
        } else if synth.osc2_waveform < 0.5 {
            // Sawtooth.
            if self.wave_t < 0.5 {
                2.0 * self.wave_t
            } else {
                2.0 * (self.wave_t - 0.5) - 1.0
            }
        } else if synth.osc2_waveform < 0.75 {
            // Square.
            if self.wave_t < 0.5 {
                1.0
            } else {
                -1.0
            }
        } else {
            // Noise.
            2.0 * self.rng_state.next_float() - 1.0
        };

        let val = (1.0 - synth.osc_balance) * osc1 + synth.osc_balance * osc2;

        // Distort.
        let max_ampl = (-10.0 * (1.0 - synth.distortion_level)).db_to_gain();
        let distorted = (val * synth.distortion_pregain.mix(-8.0, 8.0).db_to_gain())
            .clamp(-max_ampl, max_ampl);
        let val = synth.distortion_mix.mix(val, distorted);

        if synth.filter_relative {
            self.low_pass
                .set_cutoff((self.pitch * synth.filter_cutoff.mixexp(1.0, 4.0)) as f64);
        } else {
            self.low_pass
                .set_cutoff(synth.filter_cutoff.mixexp(20.0, 25000.0) as f64);
        }
        self.low_pass.set_resonance(synth.filter_resonance as f64);
        let val = self.low_pass.process(val as f64) as f32;

        // Compress.
        let (val, _) = if synth.enable_compressor {
            self.compressor.process(val, val)
        } else {
            (val, val)
        };

        // Increment time.
        self.wave_t += self.pitch / synth.sample_rate;
        self.wave_t %= 1.0;
        self.t += 1.0 / synth.sample_rate;

        let val = val * 5.0;

        const EAR_SAFETY: f32 = 0.80;
        let wave = (val * volume).clamp(-EAR_SAFETY, EAR_SAFETY);
        (wave, wave)
    }

    fn notify_release(&mut self) {
        self.released = true;
        self.release_time = self.t;
    }

    fn is_done(&self, synth: &DefaultSynth) -> bool {
        if self.released {
            let dt = self.t - self.release_time;
            dt >= synth.release_time
        } else {
            false
        }
    }
}
