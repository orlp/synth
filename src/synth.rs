pub trait Synth: Send + Sync {
    type Voice: Voice<Self>;

    fn param_change(&mut self, param: u8, value: f32);
    fn notify_buffer(&mut self);
    fn step_frame(&mut self);
}

pub trait Voice<S: Synth + ?Sized>: Send + Sync {
    fn new(pitch: f32, vel: f32, synth: &mut S) -> Self;
    fn step_frame(&mut self, synth: &S) -> (f32, f32);
    fn notify_release(&mut self);
    fn is_done(&self, synth: &S) -> bool;
}
