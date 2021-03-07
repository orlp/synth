use crate::util::*;

const MAX_WINDOW_SIZE: usize = 5000;

pub struct Compressor {
    sample_rate: f32,
    ringbuffer: [f32; MAX_WINDOW_SIZE],
    ringbuffer_idx: usize,
    window_size: usize,

    threshold: f32,
    att_rate: f32,
    rel_rate: f32,
    max_gain: f32,
    clean_mix: f32,

    rms2total: f32,
    gain: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32, window_time_ms: f32) -> Self {
        let window_size = (sample_rate * window_time_ms / 1000.0) as usize;
        if window_size >= MAX_WINDOW_SIZE {
            panic!("compressor buffer size insufficient for sample rate and window size")
        }

        let mut c = Self {
            sample_rate,
            ringbuffer: [0.0; MAX_WINDOW_SIZE],
            ringbuffer_idx: 0,
            window_size,

            threshold: 0.7,
            att_rate: 0.0, // Initialized later.
            rel_rate: 0.0, // Initialized later.
            max_gain: 0.0, // Initialized later.
            clean_mix: 0.0,

            rms2total: 0.0,
            gain: 2.0,
        };

        c.set_attack_time(20.0);
        c.set_release_time(20.0);
        c.set_max_gain(-6.0);

        c
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    /// Set the attack time in ms/6db.
    pub fn set_attack_time(&mut self, att_time_ms: f32) {
        self.att_rate = 2.0f32.powf(-1.0 / (att_time_ms / 1000.0 * self.sample_rate));
    }

    /// Set the release time in ms/6db.
    pub fn set_release_time(&mut self, rel_time_ms: f32) {
        self.rel_rate = 2.0f32.powf(1.0 / (rel_time_ms / 1000.0 * self.sample_rate));
    }

    /// Set the maximum gain in db.
    pub fn set_max_gain(&mut self, max_gain: f32) {
        self.max_gain = 2.0f32.powf(-max_gain / 6.0);
    }

    pub fn set_clean_mix(&mut self, clean_mix: f32) {
        self.clean_mix = clean_mix;
    }

    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let sqvol = l * l + r * r;
        self.rms2total -= self.ringbuffer[self.ringbuffer_idx];
        self.rms2total += sqvol;
        self.ringbuffer[self.ringbuffer_idx] = sqvol;
        self.ringbuffer_idx = (self.ringbuffer_idx + 1) % self.window_size;

        let rms2 = self.rms2total / self.window_size as f32;
        self.gain *= if rms2 > self.threshold {
            self.att_rate
        } else {
            self.rel_rate
        };
        self.gain = self.gain.clamp(0.0, self.max_gain);

        let cl = self.clean_mix.mix(l * self.gain, l);
        let cr = self.clean_mix.mix(r * self.gain, r);
        (cl, cr)
    }
}
