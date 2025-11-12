use crate::{Config, Sample, pitch::SamplePitch};

pub struct Osc<Wave, Pitch> {
    phase: f32,
    pitch: Pitch,
    _wave: Wave,
}

pub trait OscExt: Sized {
    fn sin(self) -> Osc<Sine, Self> {
        Osc {
            phase: 0.0,
            pitch: self,
            _wave: Sine,
        }
    }

    fn triangle(self) -> Osc<Triangle, Self> {
        Osc {
            phase: 0.0,
            pitch: self,
            _wave: Triangle,
        }
    }

    fn square(self) -> Osc<Square, Self> {
        Osc {
            phase: 0.0,
            pitch: self,
            _wave: Square,
        }
    }
}

impl<T> OscExt for T where T: SamplePitch {}

pub struct Sine;

impl<Pitch> Sample for Osc<Sine, Pitch>
where
    Pitch: SamplePitch,
{
    fn sample(&mut self, samples: &mut [f32], config: &Config) {
        lidsp::sine(
            samples,
            config.sample_rate,
            config.channels,
            &mut self.phase,
            // TODO: for each sample
            || self.pitch.sample_pitch(config).unwrap_or_default(),
        );
    }
}

pub struct Triangle;

impl<Pitch> Sample for Osc<Triangle, Pitch>
where
    Pitch: SamplePitch,
{
    fn sample(&mut self, samples: &mut [f32], config: &Config) {
        lidsp::triangle(
            samples,
            config.sample_rate,
            config.channels,
            &mut self.phase,
            // TODO: for each sample
            || self.pitch.sample_pitch(config).unwrap_or_default(),
        );
    }
}

pub struct Square;

impl<Pitch> Sample for Osc<Square, Pitch>
where
    Pitch: SamplePitch,
{
    fn sample(&mut self, samples: &mut [f32], config: &Config) {
        lidsp::square(
            samples,
            config.sample_rate,
            config.channels,
            &mut self.phase,
            // TODO: for each sample
            || self.pitch.sample_pitch(config).unwrap_or_default(),
            0.5,
        );
    }
}
