use crate::{Process, ops::An};

#[derive(Clone, Copy)]
pub struct Lpf<Src> {
    src: Src,
    state: f64,
    rc: f64,
}

pub trait LpfExt: Process + Sized {
    fn lpf(self, cutoff_hz: f32) -> An<Lpf<Self>> {
        An(Lpf {
            src: self,
            state: 0.0,
            rc: 1.0 / (core::f64::consts::TAU * cutoff_hz as f64),
        })
    }
}

impl<T> LpfExt for T where T: Process {}

impl<Src> Process for Lpf<Src>
where
    Src: Process,
{
    fn sample(&mut self, config: &crate::Config) -> f32 {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        let dt = config.sample_duration;
        let cf = dt / (self.rc + dt);
        let sample = self.src.sample(config) as f64;
        self.state += cf * (sample - self.state);
        self.state as f32
    }
}

#[derive(Clone, Copy)]
pub struct FadeIn<Src> {
    src: Src,
    t: f64,
    duration: f64,
}

pub trait FadeInExt: Process + Sized {
    fn fadein(self, duration: f32) -> An<FadeIn<Self>> {
        An(FadeIn {
            src: self,
            t: 0.0,
            duration: duration as f64,
        })
    }
}

impl<T> FadeInExt for T where T: Process {}

impl<Src> Process for FadeIn<Src>
where
    Src: Process,
{
    fn reset(&mut self) {
        self.t = 0.0;
    }

    fn sample(&mut self, config: &crate::Config) -> f32 {
        // https://github.com/SamiPerttu/fundsp/blob/master/src/dynamics.rs#L278
        let sample = self.src.sample(config);
        if self.t < self.duration {
            let x = self.t / self.duration;
            let value = ((x * 6.0 - 15.0) * x + 10.0) * x * x * x;
            self.t += config.sample_duration;
            sample * value as f32
        } else {
            sample
        }
    }
}
