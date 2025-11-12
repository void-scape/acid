use crate::{Config, Process, envin, ops::An};

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

// TODO: why doesn't this work?
// pub fn sin<Freq: Process>(freq: Freq) -> An<impl Process> {
//     An(envin(
//         |t, f| (t * f as f64 * core::f64::consts::TAU).sin() as f32,
//         freq,
//     ))
// }

pub fn saw<Freq: Process>(freq: Freq) -> An<impl Process> {
    An(envin(|t, f| ((t * f as f64) % 1.0) as f32, freq))
}
