use anyhow::Result;
use std::sync::mpsc;

use midir::MidiInputConnection;
use midly::live::LiveEvent;
use midly::MidiMessage;

#[derive(Copy, Clone, Debug)]
pub enum EventContent {
    NoteOff { key: u8, vel: u8 },
    NoteOn { key: u8, vel: u8 },
    Controller { controller: u8, value: u8 },
}

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub timestamp: u64,
    pub channel: u8,
    pub content: EventContent,
}

pub fn list_devices() -> Vec<String> {
    vec![]
}

pub struct Connection(MidiInputConnection<mpsc::SyncSender<Event>>);

pub fn connect_to_ports(
    midi_ports: Vec<String>,
) -> Result<(mpsc::Receiver<Event>, Vec<Connection>)> {
    let (sender, receiver) = mpsc::sync_channel(1024);

    let connections: Result<Vec<Connection>> = midi_ports
        .into_iter()
        .map(|port_name| {
            let midi_in = midir::MidiInput::new(&format!("synth to {}", port_name))?;

            let selected_port = midi_in
                .ports()
                .into_iter()
                .find(|p| midi_in.port_name(&p) == Ok(port_name.clone()))
                .expect(&format!("could not find MIDI port {}", port_name));

            let connect_result = midi_in.connect(
                &selected_port,
                &format!("synth conn to {}", port_name),
                |timestamp, bytes, sender| {
                    let midly_event = LiveEvent::parse(bytes);
                    match midly_event {
                        Ok(LiveEvent::Midi { channel, message }) => {
                            let content = match message {
                                MidiMessage::NoteOff { key, vel } => EventContent::NoteOff {
                                    key: key.into(),
                                    vel: vel.into(),
                                },

                                MidiMessage::NoteOn { key, vel } => EventContent::NoteOn {
                                    key: key.into(),
                                    vel: vel.into(),
                                },

                                MidiMessage::Controller { controller, value } => {
                                    EventContent::Controller {
                                        controller: controller.into(),
                                        value: value.into(),
                                    }
                                }

                                _ => return,
                            };

                            let send_result = sender.send(Event {
                                timestamp,
                                channel: channel.into(),
                                content,
                            });

                            if let Err(err) = send_result {
                                eprintln!("failed to send MIDI event, error: {:?}", err);
                            }
                        }

                        Err(err) => {
                            eprintln!("midly failed to parse {:?}, error: {:?}", bytes, err);
                        }

                        _ => {}
                    }
                },
                sender.clone(),
            );

            match connect_result {
                Ok(conn) => Ok(Connection(conn)),
                Err(err) => Err(err.into()),
            }
        })
        .collect();

    connections.map(|conn| (receiver, conn))
}
