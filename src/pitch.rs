use crate::{Config, trig::Trigger};

pub trait SamplePitch {
    fn sample_pitch(&mut self, config: &Config) -> Option<f32>;
}

pub struct Pitch<Src> {
    src: Src,
    pitch: f32,
}

pub trait PitchExt: Sized {
    fn pitch(self, pitch: f32) -> Pitch<Self> {
        Pitch { src: self, pitch }
    }
}

impl<T> PitchExt for T where T: Trigger {}

impl<Src> SamplePitch for Pitch<Src>
where
    Src: Trigger,
{
    fn sample_pitch(&mut self, config: &Config) -> Option<f32> {
        self.src.trigger(config).then_some(self.pitch)
    }
}
