use crate::{Config, Process, ops::An};

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

pub fn expdecay(duration: f64) -> An<impl Process> {
    An(env(move |t| (-5.0 * t / duration).exp() as f32))
}
