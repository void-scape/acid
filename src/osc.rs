use crate::{Config, Process, ops::An};

pub struct Sine<Freq> {
    freq: Freq,
    phase: f64,
}

pub fn sin<Freq>(freq: Freq) -> An<Sine<Freq>>
where
    Freq: Process,
{
    An(Sine { freq, phase: 0.0 })
}

impl<Freq> Process for Sine<Freq>
where
    Freq: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let phase = self.phase;
        self.phase += self.freq.sample(config) as f64 * config.sample_duration;
        self.phase = self.phase.fract();
        (phase as f32 * core::f32::consts::TAU).sin()
    }
}

pub struct Saw<Freq> {
    freq: Freq,
    phase: f64,
}

pub fn saw<Freq>(freq: Freq) -> An<Saw<Freq>>
where
    Freq: Process,
{
    An(Saw { freq, phase: 0.0 })
}

impl<Freq> Process for Saw<Freq>
where
    Freq: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let phase = self.phase;
        self.phase += self.freq.sample(config) as f64 * config.sample_duration;
        self.phase = self.phase.fract();
        (phase % 1.0) as f32 * 2.0 - 1.0
    }
}
