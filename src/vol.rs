use crate::{Config, Process, math, ops::An};

impl<T> LinearVolumeExt for T where T: Process {}
pub trait LinearVolumeExt: Process + Sized {
    fn linear_volume(mut self, volume: f32) -> An<impl Process> {
        An(move |config: &Config| {
            let sample = self.sample(config);
            math::clamp(sample * volume)
        })
    }
}
