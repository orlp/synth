use std::error::Error;


use cpal::traits::*;

use crate::synth::Synth;
use crate::synth_controller::SynthController;


pub fn run<T, S: Synth + 'static>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut synth: S,
    mut synth_controller: SynthController<S>,
) -> Result<cpal::Stream, Box<dyn Error>>
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            synth_controller.pump_events(&mut synth);
            synth.notify_buffer();

            for frame in data.chunks_mut(channels) {
                synth.step_frame();
                let value = synth_controller.step_all_voices(&mut synth);
                for sample in frame.iter_mut() {
                    *sample = cpal::Sample::from(&value);
                }
            }
        },
        err_fn,
    )?;
    stream.play()?;

    Ok(stream)
}
