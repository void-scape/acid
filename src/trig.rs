use crate::Config;

pub trait Trigger {
    fn trigger(&mut self, config: &Config) -> bool;
}

pub struct Step<const STEPS: usize> {
    steps: [bool; STEPS],
    current: usize,
    sample_accum: f32,
}

pub fn step<const STEPS: usize>(steps: [bool; STEPS]) -> Step<STEPS> {
    Step {
        steps,
        current: 0,
        sample_accum: 0.0,
    }
}

impl<const STEPS: usize> Trigger for Step<STEPS> {
    fn trigger(&mut self, config: &Config) -> bool {
        self.sample_accum += 1.0;
        let trigger = self.steps[self.current];
        if self.sample_accum >= config.spb {
            self.sample_accum -= config.spb;
            self.current = (self.current + 1) % STEPS;
        }
        trigger
    }
}
