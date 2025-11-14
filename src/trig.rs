use crate::{Config, Process, ops::An};

pub fn step<const LEN: usize, Src>(mut steps: [Src; LEN]) -> An<impl FnMut(&Config) -> f32 + Copy>
where
    Src: Copy + Process,
{
    let mut current = 0;
    An(move |config: &Config| {
        let sample = steps[current].sample(config);
        current = (current + 1) % LEN;
        sample
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

impl<Src> Seg<Src> {
    pub fn new(src: Src, seg: usize) -> Self {
        Self {
            src,
            seg: seg as f64,
            t: 0.0,
            retained: 0.0,
            init: false,
        }
    }
}

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

pub struct ReTrigger<Src, Trig> {
    src: Src,
    trig: Trig,
    triggered: bool,
}

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

impl<T> TrigExt for T where T: Process {}
pub trait TrigExt: Process + Sized {
    fn seg(self, seg: usize) -> An<Seg<Self>> {
        An(Seg::new(self, seg))
    }

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
