use crate::{Sample, pitch::SamplePitch};

pub struct Adsr<Src> {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    stage: Stage,
    current: f32,
    sample_counter: u32,
    retained: f32,
    release_start_level: f32,
    src: Src,
}

pub trait AdsrExt: Sized {
    fn adsr(self, attack: f32, decay: f32, sustain: f32, release: f32) -> Adsr<Self> {
        Adsr {
            attack,
            decay,
            sustain,
            release,
            stage: Stage::Idle,
            current: 0.0,
            sample_counter: 0,
            retained: 0.0,
            release_start_level: 0.0,
            src: self,
        }
    }
}

impl<T> AdsrExt for T {}

impl<Src> Adsr<Src> {
    pub fn attack(&mut self) {
        self.stage = Stage::Attack;
        self.sample_counter = 0;
    }

    pub fn release(&mut self) {
        self.release_start_level = self.current;
        self.stage = Stage::Release;
        self.sample_counter = 0;
    }

    pub fn process(&mut self, sample_rate: f32, pitch: Option<f32>) -> f32 {
        let is_on = pitch.is_some();
        let is_off = pitch.is_none();

        if is_on && matches!(self.stage, Stage::Idle) {
            self.attack();
        } else if is_off && !matches!(self.stage, Stage::Release | Stage::Idle) {
            self.release();
        }

        if let Some(pitch) = pitch {
            self.retained = pitch;
        }

        match self.stage {
            Stage::Idle => {
                self.current = 0.0;
            }
            Stage::Attack => {
                let attack_samples = (self.attack * sample_rate).max(1.0) as u32;
                if self.sample_counter < attack_samples {
                    let progress = self.sample_counter as f32 / attack_samples as f32;
                    self.current = progress * progress;
                    self.sample_counter += 1;
                } else {
                    self.current = 1.0;
                    self.stage = Stage::Decay;
                    self.sample_counter = 0;
                }
            }
            Stage::Decay => {
                let decay_samples = (self.decay * sample_rate).max(1.0) as u32;
                if self.sample_counter < decay_samples {
                    let progress = self.sample_counter as f32 / decay_samples as f32;
                    let curve = (-4.0 * progress).exp();
                    self.current = self.sustain + (1.0 - self.sustain) * curve;
                    self.sample_counter += 1;
                } else {
                    self.current = self.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => {
                self.current = self.sustain;
            }
            Stage::Release => {
                let release_samples = (self.release * sample_rate).max(1.0) as u32;
                if self.sample_counter < release_samples {
                    let progress = self.sample_counter as f32 / release_samples as f32;
                    let curve = (-4.0 * progress).exp();
                    self.current = self.release_start_level * curve;
                    self.sample_counter += 1;
                } else {
                    self.current = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }

        self.current
    }
}

enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl<Src> Sample for Adsr<Src>
where
    Src: Sample,
{
    fn sample(&mut self, samples: &mut [f32], config: &crate::Config) {
        self.src.sample(samples, config);
        for sample in samples.iter_mut() {
            let adsr_sample = (*sample > f32::EPSILON).then_some(*sample);
            let adsr = self.process(config.sample_rate, adsr_sample);
            *sample *= adsr;
        }
    }
}

impl<Src> SamplePitch for Adsr<Src>
where
    Src: SamplePitch,
{
    fn sample_pitch(&mut self, config: &crate::Config) -> Option<f32> {
        let pitch = self.src.sample_pitch(config);
        let adsr = self.process(config.sample_rate, pitch);
        (adsr > f32::EPSILON).then_some(adsr * self.retained)
    }
}
