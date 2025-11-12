use crate::{Config, Process, ops::An};

pub fn samples(samples: usize) -> An<impl Process> {
    let mut csample = 0;
    An(move |_: &Config| {
        let sample = if csample >= samples {
            csample = 0;
            1.0
        } else {
            0.0
        };
        csample += 1;
        sample
    })
}

pub fn beats(beats: usize) -> An<impl Process> {
    let mut sample = 0.0;
    let mut cbeat = 0;
    An(move |config: &Config| {
        sample += 1.0;
        if sample >= config.spb {
            sample -= config.spb;
            cbeat += 1;
        }
        if cbeat >= beats {
            cbeat -= beats;
            1.0
        } else {
            0.0
        }
    })
}

#[derive(Clone, Copy)]
pub struct Seg<Src> {
    src: Src,
    seg: f64,
    t: f64,
    retained: f32,
    init: bool,
}

pub trait SegExt: Process + Sized {
    fn seg(self, seg: usize) -> An<Seg<Self>> {
        An(Seg {
            src: self,
            seg: seg as f64,
            t: 0.0,
            retained: 0.0,
            init: false,
        })
    }
}

impl<T> SegExt for T where T: Process {}

impl<Src> Process for Seg<Src>
where
    Src: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let duration = config.spb / self.seg;
        if !self.init || self.t >= duration {
            self.init = true;
            self.t -= duration;
            self.retained = self.src.sample(config);
        }
        self.t += 1.0;
        self.retained
    }
}

#[derive(Clone, Copy)]
pub struct Step<const STEPS: usize> {
    steps: [f32; STEPS],
    current: usize,
}

pub fn step<const STEPS: usize>(steps: [f32; STEPS]) -> An<Step<STEPS>> {
    An(Step { steps, current: 0 })
}

impl<const STEPS: usize> Process for Step<STEPS> {
    fn sample(&mut self, _: &Config) -> f32 {
        let sample = self.steps[self.current];
        self.current = (self.current + 1) % STEPS;
        sample
    }
}

pub struct ReTrigger<Src, Trig> {
    src: Src,
    trig: Trig,
    triggered: bool,
}

pub trait ReTriggerExt: Process + Sized {
    fn retrig<Trig>(self, trigger: Trig) -> An<ReTrigger<Self, Trig>>
    where
        Trig: Process,
    {
        An(ReTrigger {
            src: self,
            trig: trigger,
            triggered: false,
        })
    }
}

impl<T> ReTriggerExt for T where T: Process {}

impl<Src, Trig> Process for ReTrigger<Src, Trig>
where
    Src: Process,
    Trig: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let trigger = self.trig.sample(config);
        if !self.triggered && trigger > 0.0 {
            self.triggered = true;
            self.src.reset();
        } else if self.triggered && trigger <= 0.0 {
            self.triggered = false;
        }
        self.src.sample(config)
    }
}
