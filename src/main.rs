#![allow(unused_imports, dead_code)]

use anyhow::Result;
use std::sync::mpsc;
use util::log_if_error;

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

#[derive(StructOpt, Debug)]
struct PlayOpt {
    #[structopt(short = "k", long = "keyboard")]
    /// The MIDI channel the synthesizer listens on for keyboard events.
    midi_keyboard_channel: u8,

    #[structopt(short = "c", long = "controller")]
    /// The MIDI channel the synthesizer listens on for controller events.
    midi_controller_channel: u8,

    #[structopt(short = "o", long = "output-device", number_of_values = 1)]
    /// The audio output devices.
    output_devices: Vec<String>,

    /// Input midi ports.
    input_midi_ports: Vec<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Tools for working with midi.")]
enum SynthOpt {
    /// List all available MIDI ports.
    ListMIDI,

    /// List all available audio devices.
    ListAudio,

    /// Play the software synth.
    Play(PlayOpt),
}

fn play(opt: PlayOpt) -> Result<()> {
    // let button_map = ButtonMap::from_toml()?;
    let host = cpal::default_host();

    println!("{:?}", opt);

    let output_devices = if opt.output_devices.len() == 0 {
        vec![host
            .default_output_device()
            .expect("no output device available")]
    } else {
        opt.output_devices
            .iter()
            .map(|device_name| {
                host.devices()
                    .expect("could not get devices")
                    .filter(|device| {
                        device
                            .name()
                            .map(|n| n.to_lowercase() == device_name.to_lowercase())
                            .ok()
                            == Some(true)
                    })
                    .filter(|device| {
                        device
                            .supported_output_configs()
                            .map(|mut it| it.next().is_some())
                            .ok()
                            == Some(true)
                    })
                    .next()
                    .expect(&format!(
                        "could not find output device with name {}",
                        device_name
                    ))
            })
            .collect()
    };


    let (midi_event_queue, _midi_connections) =
        midi::connect_to_ports(opt.input_midi_ports.clone())?;

    let mut output_streams: Vec<_> = output_devices
        .into_iter()
        .map(|device| {
            let mut supported_configs_range = device
                .supported_output_configs()
                .expect("error while querying configs");

            let supported_config = supported_configs_range
                .next()
                .expect("no supported config")
                .with_max_sample_rate();

            let sample_format = supported_config.sample_format();
            let config: cpal::StreamConfig = supported_config.into();

            let synth = synthesizers::default::DefaultSynth::new(
                "buttonmaps/bcr2000.toml",
                config.sample_rate.0 as f32,
            )
            .expect("could not create synth");

            let (kb_event_sender, kb_event_queue) = mpsc::sync_channel(1024);
            let kb_ctrlr = MidiController::new(
                kb_event_sender,
                opt.midi_keyboard_channel,
                opt.midi_controller_channel,
            );
            let synth_ctrlr = SynthController::new(kb_event_queue);

            let stream = match sample_format {
                SampleFormat::F32 => audio::run::<f32, _>(&device, &config, synth, synth_ctrlr),
                SampleFormat::I16 => audio::run::<i16, _>(&device, &config, synth, synth_ctrlr),
                SampleFormat::U16 => audio::run::<u16, _>(&device, &config, synth, synth_ctrlr),
            }
            .unwrap();

            (kb_ctrlr, stream)
        })
        .collect();

    loop {
        let event = midi_event_queue.recv().unwrap();
        for output_stream in output_streams.iter_mut() {
            output_stream.0.handle_midi_event(event);
        }
    }
}

fn main() -> Result<()> {
    let opt = SynthOpt::from_args();

    match opt {
        SynthOpt::ListMIDI => {
            println!("Available devices:");
            for device_name in midi::list_devices() {
                println!("{}", device_name);
            }
        }

        SynthOpt::ListAudio => {
            let host = cpal::default_host();

            println!("Available output devices:");
            for device in host.devices()? {
                let is_output = log_if_error(
                    "error while getting device supported output configs",
                    device.supported_output_configs(),
                )
                .map(|mut it| it.next().is_some())
                .unwrap_or(false);

                if is_output {
                    println!(
                        "{}",
                        device.name().unwrap_or("error while reading name".into())
                    );
                }
            }
        }

        SynthOpt::Play(playopt) => {
            return play(playopt);
        }
    }

    Ok(())
}
