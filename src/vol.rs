use crate::{Config, Sample};

pub struct LinearVolume<Src> {
    src: Src,
    volume: f32,
}

pub trait LinearVolumeExt: Sized {
    fn linear_volume(self, volume: f32) -> LinearVolume<Self> {
        LinearVolume { src: self, volume }
    }
}

impl<T> LinearVolumeExt for T where T: Sample {}

impl<Src> Sample for LinearVolume<Src>
where
    Src: Sample,
{
    fn sample(&mut self, samples: &mut [f32], config: &Config) {
        self.src.sample(samples, config);
        lidsp::linear_volume(samples, self.volume);
    }
}
