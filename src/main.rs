#![allow(unused_imports, dead_code)]

use std::error::Error;
use std::sync::mpsc;

use cpal::traits::*;
use cpal::SampleFormat;
use structopt::StructOpt;

mod audio;
mod midi;
mod midi_controller;
mod synth;
mod synth_controller;
mod synthesizers;
mod util;

use midi_controller::MidiController;
use synth_controller::SynthController;


#[derive(StructOpt)]
struct PlayOpt {
    #[structopt(short = "k", long = "keyboard")]
    /// The MIDI channel the synthesizer listens on for keyboard events.
    midi_keyboard_channel: u8,

    #[structopt(short = "c", long = "controller")]
    /// The MIDI channel the synthesizer listens on for controller events.
    midi_controller_channel: u8,

    input_midi_ports: Vec<String>,
}

#[derive(StructOpt)]
#[structopt(about = "Tools for working with midi.")]
enum SynthOpt {
    /// List all available MIDI ports.
    ListMIDI,

    /// Play the software synth.
    Play(PlayOpt),
}


fn play(opt: PlayOpt) -> Result<(), Box<dyn Error>> {
    // let button_map = ButtonMap::from_toml()?;
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();


    let synth = synthesizers::default::DefaultSynth::new(
        "buttonmaps/bcr2000.toml",
        config.sample_rate.0 as f32,
    )?;

    let (midi_event_queue, _midi_connections) = midi::connect_to_ports(opt.input_midi_ports)?;
    let (kb_event_sender, kb_event_queue) = mpsc::sync_channel(1024);
    let mut kb_ctrlr = MidiController::new(
        kb_event_sender,
        opt.midi_keyboard_channel,
        opt.midi_controller_channel,
    );
    let synth_ctrlr = SynthController::new(kb_event_queue);


    let _stream = &match sample_format {
        SampleFormat::F32 => audio::run::<f32, _>(&device, &config, synth, synth_ctrlr),
        SampleFormat::I16 => audio::run::<i16, _>(&device, &config, synth, synth_ctrlr),
        SampleFormat::U16 => audio::run::<u16, _>(&device, &config, synth, synth_ctrlr),
    }
    .unwrap();

    loop {
        let event = midi_event_queue.recv().unwrap();
        kb_ctrlr.handle_midi_event(event);
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let opt = SynthOpt::from_args();

    match opt {
        SynthOpt::ListMIDI => {
            println!("Available devices:");
            for device_name in midi::list_devices() {
                println!("{}", device_name);
            }
        }

        SynthOpt::Play(playopt) => {
            return play(playopt);
        }
    }

    Ok(())
}
