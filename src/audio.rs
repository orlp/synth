use anyhow::Result;


use cpal::traits::*;

use crate::synth::Synth;
use crate::synth_controller::SynthController;


pub fn run<T, S: Synth + 'static>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut synth: S,
    mut synth_controller: SynthController<S>,
) -> Result<cpal::Stream>
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


            if channels == 1 {
                for sample in data.iter_mut() {
                    synth.step_frame();
                    let (l, r) = synth_controller.step_all_voices(&mut synth);
                    *sample = cpal::Sample::from(&((l + r) / 2.0));
                }
            } else if channels == 2 {
                for frame in data.chunks_mut(2) {
                    synth.step_frame();
                    let (l, r) = synth_controller.step_all_voices(&mut synth);
                    frame[0] = cpal::Sample::from(&l);
                    frame[1] = cpal::Sample::from(&r);
                }
            } else {
                panic!("can't output to more than 2 channels");
            }
        },
        err_fn,
    )?;
    stream.play()?;

    Ok(stream)
}
