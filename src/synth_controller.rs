use slotmap::{DefaultKey, DenseSlotMap, Key};
use std::sync::mpsc;

use crate::synth::{Synth, Voice};

const MAX_CHANNELS: usize = 64;

#[derive(Copy, Clone, Debug)]
pub enum SynthEvent {
    NoteOn { key: u8, vel: f32 },
    NoteOff { key: u8 },
    ParamChange { param: u8, value: f32 },
}

#[derive(Debug)]
struct Channel<S: Synth> {
    key: u8,
    voice: S::Voice,
    is_sustained: bool,
    id: u64,
}

pub struct SynthController<S: Synth> {
    event_queue: mpsc::Receiver<SynthEvent>,
    channels: DenseSlotMap<DefaultKey, Channel<S>>,
    sustained_voices: [DefaultKey; 128],
    num_sustained_voices: usize,
    id_ctr: u64,
}

impl<S: Synth> SynthController<S> {
    pub fn new(event_queue: mpsc::Receiver<SynthEvent>) -> Self {
        Self {
            event_queue,
            channels: DenseSlotMap::with_capacity(MAX_CHANNELS + 1),
            sustained_voices: [DefaultKey::null(); 128],
            num_sustained_voices: 0,
            id_ctr: 0,
        }
    }

    fn add_channel(&mut self, key: u8, voice: S::Voice) -> DefaultKey {
        let channel = Channel {
            key,
            voice,
            is_sustained: true,
            id: self.id_ctr,
        };
        self.id_ctr = self.id_ctr.wrapping_add(1);

        if self.channels.len() == MAX_CHANNELS {
            let (oldest_key, _) = self
                .channels
                .iter()
                .max_by(|(_, c1), (_, c2)| {
                    let x = (c1.is_sustained, c1.id);
                    let y = (c2.is_sustained, c2.id);
                    x.cmp(&y)
                })
                .unwrap();

            self.remove_channel(oldest_key);
        }

        let channel_key = self.channels.insert(channel);
        self.sustained_voices[key as usize] = channel_key;
        self.num_sustained_voices += 1;
        channel_key
    }

    fn remove_channel(&mut self, key: DefaultKey) -> Option<Channel<S>> {
        self.channels.remove(key).map(|c| {
            if c.is_sustained {
                self.sustained_voices[c.key as usize] = DefaultKey::null();
                self.num_sustained_voices -= 1;
            }
            c
        })
    }

    pub fn pump_events(&mut self, synth: &mut S) {
        // Only keep channels that play voices that aren't done yet.
        self.channels.retain(|_k, c| !c.voice.is_done());

        while let Ok(event) = self.event_queue.try_recv() {
            match event {
                SynthEvent::NoteOn { key, vel } => {
                    let pitch = 440.0 * 2.0f32.powf((key as f32 - 69.0) / 12.0);
                    self.add_channel(key, Voice::new(pitch, vel, synth));
                }

                SynthEvent::NoteOff { key } => {
                    if let Some(c) = self.channels.get_mut(self.sustained_voices[key as usize]) {
                        c.is_sustained = false;
                        c.voice.notify_release();
                    }
                }

                SynthEvent::ParamChange { param, value } => {
                    synth.param_change(param, value);
                }
            }
        }
    }

    pub fn step_all_voices(&mut self, synth: &mut S) -> f32 {
        let mut value = 0.0;
        for c in self.channels.values_mut() {
            value += c.voice.step_frame(&synth);
        }
        value
    }
}
