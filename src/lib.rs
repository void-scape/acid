mod adsr;
mod filter;
mod note;
mod ops;
mod osc;
mod rng;
mod trig;
mod vol;

mod prelude {
    pub use super::filter::*;
    pub use super::osc::*;
    pub use super::rng::*;
    pub use super::trig::*;
    pub use super::vol::*;
}

use crate::ops::An;
use prelude::*;

// need to send to the audio thread in `glazer::audio_stub`
unsafe impl Send for Acid {}
macro_rules! acid {
    (#[Header] $($sound:expr),*) => {
    };
    (bpm: $bpm:expr, $($name:ident: $sound:expr),* $(,)?) => {
        pub struct Acid {
            sounds: Vec<(Box<dyn Process>, Vec<f32>)>,
        }
        impl Default for Acid {
            fn default() -> Self {
                Self {
                    sounds: vec![$((Box::new($sound), Vec::new())),*],
                }
            }
        }
        #[unsafe(no_mangle)]
        pub fn handle_input(_: glazer::PlatformInput<Acid>) {}
        #[unsafe(no_mangle)]
        pub fn update_and_render(
            glazer::PlatformUpdate {
                memory: da,
                sample_rate,
                samples,
                channels,
                ..
            }: glazer::PlatformUpdate<Acid, u32>,
        ) {
            let [$($name),*] = &mut da.sounds[..] else { unreachable!() };
            let len = samples.len();
            $(if $name.1.len() < len { $name.1.extend((0..len).map(|_| 0.0)) })*
            let config = Config {
                sample_rate: sample_rate as u32,
                sample_duration: 1.0 / sample_rate as f64,
                channels,
                bpm: $bpm,
                spb: (60.0 / ($bpm)) * sample_rate as f64
            };
            $($name.0.process(&mut $name.1[0..len], &config);)*
            lidsp::mix!(samples, $($name.1),*);
        }
    };
}

acid!(
    bpm: 140.0,
    kick: kick909().linear_volume(0.65),
    bass: bass().linear_volume(0.3),
);

fn kick909() -> impl Process {
    let beat = step([1.0, 0.0, 0.0, 0.0]).seg(4);
    let pressure = env(|t| (-t * 12.0).exp() as f32).retrig(beat) * 125.0;
    (sin(beat * 60.0 + pressure) * env(|t| (-t * 4.0).exp() as f32).retrig(beat))
        .fadein(0.002)
        .retrig(beat)
}

fn bass() -> impl Process {
    let cminor = [
        261.63, 293.66, 311.13, 349.23, 392.00, 415.30, 466.16, 523.25,
    ];
    let notes = slist(cminor, ru32(69).retrig(samples(16)).seg(4));
    saw(notes / 2.0).lpf(1500.0)
}

/// Maps `src` [`Process::sample`] to an index into `list`. Assumes samples
/// are in the range `0.0..1.0`.
fn slist<const LEN: usize>(list: [f32; LEN], mut src: impl Process) -> An<impl Process> {
    An(move |config: &Config| {
        let index = (src.sample(config) * list.len() as f32) as usize;
        list[index]
    })
}

pub struct Config {
    pub sample_rate: u32,
    pub sample_duration: f64,
    pub channels: usize,
    pub bpm: f64,
    pub spb: f64,
}

pub trait Process {
    fn reset(&mut self) {}
    fn sample(&mut self, config: &Config) -> f32;
    fn process(&mut self, samples: &mut [f32], config: &Config) {
        debug_assert!(samples.len().is_multiple_of(config.channels));
        for frame in samples.chunks_mut(config.channels) {
            frame.fill(self.sample(config));
        }
    }
}

impl Process for f32 {
    fn sample(&mut self, _: &Config) -> f32 {
        *self
    }
}

impl<T> Process for T
where
    T: FnMut(&Config) -> f32,
{
    fn sample(&mut self, config: &Config) -> f32 {
        self(config)
    }
}

pub struct Env<F> {
    f: F,
    t: f64,
}

pub fn env<F: FnMut(f64) -> f32>(f: F) -> An<Env<F>> {
    An(Env { f, t: 0.0 })
}

impl<F> Process for Env<F>
where
    F: FnMut(f64) -> f32,
{
    fn reset(&mut self) {
        self.t = 0.0;
    }

    fn sample(&mut self, config: &Config) -> f32 {
        let env = (self.f)(self.t);
        self.t += config.sample_duration;
        env
    }
}

pub struct EnvIn<F, Src> {
    f: F,
    t: f64,
    src: Src,
}

pub fn envin<F: FnMut(f64, f32) -> f32, Src: Process>(f: F, src: Src) -> An<EnvIn<F, Src>> {
    An(EnvIn { f, t: 0.0, src })
}

impl<F, Src> Process for EnvIn<F, Src>
where
    F: FnMut(f64, f32) -> f32,
    Src: Process,
{
    fn reset(&mut self) {
        self.t = 0.0;
        self.src.reset();
    }

    fn sample(&mut self, config: &Config) -> f32 {
        let env = (self.f)(self.t, self.src.sample(config));
        self.t += config.sample_duration;
        env
    }
}
