use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone, Deserialize)]
pub struct ButtonMap {
    pub master_volume: u8,
    pub key_velocity: u8,
    pub volume_attack: u8,
    pub volume_decay: u8,
    pub volume_sustain: u8,
    pub volume_release: u8,

    pub osc1_waveform: u8,
    pub osc2_waveform: u8,
    pub osc_balance: u8,
}

impl ButtonMap {
    pub fn from_toml(fname: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(fname)?;
        let mut file_as_string = String::new();
        file.read_to_string(&mut file_as_string)?;
        toml::from_str(&file_as_string).map_err(|e| e.into())
    }
}
