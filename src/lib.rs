mod adsr;
mod osc;
mod pitch;
mod trig;
mod vol;

mod prelude {
    pub use super::adsr::*;
    pub use super::osc::*;
    pub use super::pitch::*;
    pub use super::trig::*;
    pub use super::vol::*;
}

use prelude::*;

macro_rules! acid {
    (#[Header] $($sound:expr),*) => {
        pub struct Acid {
            sounds: Vec<(Box<dyn Sample>, Vec<f32>)>,
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
    };
    (bpm: $bpm:expr, $($name:ident: $sound:expr),* $(,)?) => {
        acid!(#[Header] $($sound),*);
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
                sample_rate,
                channels,
                bpm: $bpm,
                spb: (60.0 / ($bpm)) * sample_rate
            };
            $($name.0.sample(&mut $name.1[0..len], &config);)*
            lidsp::mix!(samples, $($name.1),*);
        }
    };
    (bpm: $bpm:expr, $sound:expr) => {
        acid!(#[Header] $sound);
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
            let config = Config {
                sample_rate,
                channels,
                bpm: $bpm,
                spb: (60.0 / ($bpm)) * sample_rate
            };
            da.sounds[0].0.sample(samples, &config);
        }
    };
}

acid!(
    bpm: 140.0 * 4.0,
    osc1: step([true, false, false, false]).pitch(255.0).sin().adsr(0.0, 0.3, 0.0, 0.1),
);

fn test() {
    // fundamental_freq = 50-65 Hz
    // pitch_start = fundamental_freq * 4 (or more)
    //
    // for each sample:
    //     pitch_env = exp(-time * pitch_decay_rate)
    //     current_pitch = fundamental_freq + (pitch_start - fundamental_freq) * pitch_env
    //
    //     amplitude_env = exp(-time * amp_decay_rate)
    //
    //     sample = sin(phase) * amplitude_env
    //     phase += 2π * current_pitch / sample_rate
}

pub struct Config {
    pub sample_rate: f32,
    pub channels: usize,
    pub bpm: f32,
    pub spb: f32,
}

pub trait Sample {
    fn sample(&mut self, samples: &mut [f32], config: &Config);
}
