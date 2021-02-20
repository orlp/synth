use std::sync::mpsc;

use crate::midi;
use crate::synth_controller::SynthEvent;
use crate::util::*;

const MIDI_SUSTAIN_PEDAL: u8 = 64;


#[derive(Clone, Debug)]
pub struct MidiController {
    sustain_pedal: bool,
    pressed: [bool; 128],
    sustained: [bool; 128],
    event_output: mpsc::SyncSender<SynthEvent>,
    keyboard_channel: u8,
    controller_channel: u8,
}


impl MidiController {
    pub fn new(
        event_output: mpsc::SyncSender<SynthEvent>,
        keyboard_channel: u8,
        controller_channel: u8,
    ) -> Self {
        Self {
            sustain_pedal: false,
            pressed: [false; 128],
            sustained: [false; 128],
            event_output,
            keyboard_channel,
            controller_channel,
        }
    }

    fn key_on(&mut self, key: u8, vel: u8) {
        let already_pressed = self.pressed[key as usize] || self.sustained[key as usize];
        if already_pressed {
            self.send_event(SynthEvent::NoteOff { key });
        }
        self.send_event(SynthEvent::NoteOn {
            key,
            vel: vel as f32 / 127.0,
        });


        self.pressed[key as usize] = true;
        if self.sustain_pedal {
            self.sustained[key as usize] = true;
        }
    }

    fn key_off(&mut self, key: u8) {
        if self.pressed[key as usize] && !self.sustain_pedal {
            self.send_event(SynthEvent::NoteOff { key });
        }

        self.pressed[key as usize] = false;
    }

    fn pedal_on(&mut self) {
        self.sustain_pedal = true;

        for i in 0..128 {
            self.sustained[i] |= self.pressed[i];
        }
    }

    fn pedal_off(&mut self) {
        self.sustain_pedal = false;

        for i in 0..128 {
            if self.sustained[i] && !self.pressed[i] {
                self.send_event(SynthEvent::NoteOff { key: i as u8 });
            }

            self.sustained[i] = false;
        }
    }

    pub fn handle_midi_event(&mut self, event: midi::Event) {
        match event.content {
            midi::EventContent::NoteOn { key, vel } => {
                if event.channel == self.keyboard_channel {
                    self.key_on(key, vel);
                }
            }

            midi::EventContent::NoteOff { key, .. } => {
                if event.channel == self.keyboard_channel {
                    self.key_off(key);
                }
            }

            midi::EventContent::Controller { controller, value } => {
                if event.channel == self.keyboard_channel && controller == MIDI_SUSTAIN_PEDAL {
                    if value > 0 {
                        self.pedal_on()
                    } else {
                        self.pedal_off()
                    }
                }

                if event.channel == self.controller_channel {
                    self.send_event(SynthEvent::ParamChange {
                        param: controller,
                        value: value as f32 / 127.0,
                    });
                }
            }
        }
    }

    fn send_event(&mut self, event: SynthEvent) {
        let r = self.event_output.try_send(event);
        log_if_error("note send_event failed", r);
    }
}
