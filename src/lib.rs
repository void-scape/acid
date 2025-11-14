mod adsr;
mod comb;
mod env;
mod filter;
mod math;
mod note;
mod ops;
mod osc;
mod rng;
mod trig;
mod vol;

mod prelude {
    // pub use super::adsr::*;
    // pub use super::ops::*;
    pub use super::comb::*;
    pub use super::env::*;
    pub use super::filter::*;
    pub use super::note::*;
    pub use super::osc::*;
    pub use super::rng::*;
    pub use super::trig::*;
}

use prelude::*;

macro_rules! mix {
    ($output:expr, $($samples:expr),+) => {
        let output_len = $output.len();
        $(debug_assert!($samples.len() >= output_len);)*
        for i in 0..output_len {
            let mut sum = 0f32;
            $(sum += $samples[i];)+
            $output[i] = sum;
        }
    }
}
// need to send to the audio thread in `glazer::audio_stub`
unsafe impl Send for Acid {}
macro_rules! acid {
    {
        bpm: $bpm:expr,
        $($name:ident: $sound:expr),* $(,)?
    } => {
        pub struct Acid {
            limiters: Vec<$crate::prelude::Limiter<()>>,
            sounds: Vec<(Box<dyn $crate::Process>, Vec<f32>)>,
        }
        impl Default for Acid {
            fn default() -> Self {
                Self {
                    limiters: Vec::new(),
                    sounds: build_sounds(),
                }
            }
        }
        fn build_sounds() -> Vec<(Box<dyn $crate::Process>, Vec<f32>)> {
            vec![$((Box::new($sound), Vec::new())),*]
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
                frame_buffer,
                width,
                height,
                reloaded,
                ..
            }: glazer::PlatformUpdate<Acid, u32>,
        ) {
            for y in 0..height as u32 {
                for x in 0..width as u32 {
                    let red = x % 255;
                    let green = y % 255;
                    let blue = (x * y) % 255;
                    let index = y as usize * width + x as usize;
                    frame_buffer[index] = blue | (green << 8) | (red << 16);
                }
            }
            if reloaded {
                da.sounds = build_sounds();
            }
            let [$($name),*] = &mut da.sounds[..] else { unreachable!() };
            let len = samples.len();
            $(if $name.1.len() < len { $name.1.extend((0..len).map(|_| 0.0)) })*
            let config = Config {
                sample_rate: sample_rate as f64,
                sample_duration: 1.0 / sample_rate as f64,
                channels,
                bpm: $bpm,
                spb: (60.0 / ($bpm)) * sample_rate as f64,
                bps: 1.0 / ((60.0 / ($bpm)) * sample_rate as f64),
            };
            $($name.0.process(&mut $name.1[0..len], &config);)*
            while da.limiters.len() < channels {
                da.limiters.push(
                    $crate::prelude::Limiter::new(
                        (),
                        $crate::ms(1.0),
                        $crate::ms(500.0),
                        sample_rate as f64,
                    )
                );
            }
            mix!(samples, $($name.1),*);
            for frame in samples.chunks_mut(channels) {
                for (sample, limiter) in frame.iter_mut().zip(da.limiters.iter_mut()) {
                    *sample = $crate::math::clamp(limiter.limit(*sample));
                }
            }
        }
    };
}

acid!(
    bpm: 140.0,
    kick: kick909(),
    bass: bass(),
);

fn kick909() -> impl Process {
    let beat = step([1.0, 0.0, 0.0, 0.0]).seg(4);
    let pressure = env(|t| ((-t * 16.0).exp() * 125.0) as f32).retrig(beat);
    sin(pressure + 50.0).retrig(beat).fadein(0.005).retrig(beat)
        * env(|t| (-t * 12.0).exp() as f32).retrig(beat)
        * 0.5
}

fn bass() -> impl Process {
    let seed = 24;
    let notes =
        ru32(seed).retrig(samples(16)).seg(4).dphrydom() * step([1.0, 2.0, 1.0, 2.0, 2.0]).seg(1);
    saw(notes / 2.0)
        .lpf(sin(0.1) * 20.0 + 5000.0)
        .depth(2.0)
        .q(12.0)
        .env(expdecay(0.1).retrig(samples(16)).seg(4))
        * 0.1
}

pub struct Config {
    pub sample_rate: f64,
    pub sample_duration: f64,
    pub channels: usize,
    pub bpm: f64,
    pub spb: f64,
    pub bps: f64,
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

pub fn ms(ms: f32) -> f32 {
    ms / 1_000.0
}
