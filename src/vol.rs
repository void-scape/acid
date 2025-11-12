use crate::{Config, Process};

pub struct LinearVolume<Src> {
    src: Src,
    volume: f32,
}

pub trait LinearVolumeExt: Sized {
    fn linear_volume(self, volume: f32) -> LinearVolume<Self> {
        LinearVolume { src: self, volume }
    }
}

impl<T> LinearVolumeExt for T where T: Process {}

impl<Src> Process for LinearVolume<Src>
where
    Src: Process,
{
    fn sample(&mut self, _: &Config) -> f32 {
        self.volume
    }

    fn process(&mut self, samples: &mut [f32], config: &Config) {
        self.src.process(samples, config);
        lidsp::linear_volume(samples, self.volume);
    }
}
