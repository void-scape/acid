use crate::{Process, ops::An};

pub struct LiveAdsr<Src> {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    stage: Stage,
    current: f32,
    sample_counter: u32,
    release_start: f32,
    src: Src,
}

enum Stage {
    Idle,
    Ads,
    Release,
}

pub trait LiveAdsrExt: Process + Sized {
    fn adsr(self, attack: f32, decay: f32, sustain: f32, release: f32) -> An<LiveAdsr<Self>> {
        An(LiveAdsr {
            attack,
            decay,
            sustain,
            release,
            stage: Stage::Idle,
            current: 0.0,
            sample_counter: 0,
            release_start: 0.0,
            src: self,
        })
    }
}

impl<T> LiveAdsrExt for T where T: Process {}
impl<Src> LiveAdsr<Src> {
    pub fn process(&mut self, sample_rate: f32, gate: f32) -> f32 {
        if matches!(self.stage, Stage::Idle) && gate > 0.0 {
            self.stage = Stage::Ads;
            self.sample_counter = 0;
        } else if matches!(self.stage, Stage::Ads) && gate <= 0.0 {
            self.stage = Stage::Release;
            self.release_start = self.current;
            self.sample_counter = 0;
        }

        match self.stage {
            Stage::Idle => {}
            Stage::Ads => {
                let attack_samples = (self.attack * sample_rate) as u32;
                self.current = if self.attack > 0.0 && self.sample_counter < attack_samples {
                    let t = self.sample_counter as f32 / attack_samples as f32;
                    self.sample_counter += 1;
                    lerp(0.0, 1.0, t)
                } else {
                    let decay_samples = (self.decay * sample_rate) as u32;
                    if self.decay > 0.0 && self.sample_counter < attack_samples + decay_samples {
                        let t =
                            (self.sample_counter - attack_samples) as f32 / decay_samples as f32;
                        self.sample_counter += 1;
                        lerp(1.0, self.sustain, t)
                    } else {
                        self.sustain
                    }
                };
            }
            Stage::Release => {
                let release_samples = (self.release * sample_rate).max(1.0) as u32;
                let t = self.sample_counter as f32 / release_samples as f32;
                if self.sample_counter < release_samples {
                    self.current = lerp(self.release_start, 0.0, t);
                    self.sample_counter += 1;
                } else {
                    self.stage = Stage::Idle;
                    self.current = 0.0;
                }
            }
        }

        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            b * t + (1.0 - t) * a
        }

        debug_assert!(self.current >= 0.0 && self.current <= 1.0);
        self.current
    }
}

impl<Src> Process for LiveAdsr<Src>
where
    Src: Process,
{
    fn sample(&mut self, config: &crate::Config) -> f32 {
        let gate = self.src.sample(config);
        self.process(config.sample_rate as f32, gate)
    }
}
