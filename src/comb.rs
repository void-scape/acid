use crate::{Config, Process, ops::An};

pub fn c(c: f32) -> An<f32> {
    An(c)
}

pub fn count(n: usize) -> An<impl Process> {
    let mut x = 0;
    An(move |_: &Config| {
        let xx = x;
        x += 1;
        if x > n {
            x = 0;
        }
        xx as f32
    })
}

pub fn count_by(n: usize, mut f: impl FnMut(&Config) -> f64) -> An<impl Process> {
    let n = n as f64;
    let mut x = 0.0;
    An(move |config: &Config| {
        let xx = x;
        x += f(config);
        if x > n {
            x -= n;
        }
        xx as f32
    })
}

pub fn samples(samples: usize) -> An<impl Process> {
    An(count(samples).map(move |c| (c as usize == samples) as u32 as f32))
}

pub fn beats(beats: usize) -> An<impl Process> {
    An(count_by(beats, |c| c.bps).map(move |c| (c as usize == beats) as u32 as f32))
}

pub struct Init<F, T> {
    f: Option<F>,
    t: Option<T>,
}

pub fn init<F, T>(f: F) -> An<Init<F, T>>
where
    F: FnOnce(&Config) -> T,
    T: Process,
{
    An(Init {
        f: Some(f),
        t: None,
    })
}

impl<F, T> Process for Init<F, T>
where
    F: FnOnce(&Config) -> T,
    T: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let t = self
            .t
            .get_or_insert_with(|| (self.f.take().unwrap())(config));
        t.sample(config)
    }
}

impl<T> CombExt for T where T: Process {}
pub trait CombExt: Process + Sized {
    fn map<F>(mut self, mut f: F) -> An<impl Process>
    where
        F: FnMut(f32) -> f32,
    {
        An(move |config: &Config| f(self.sample(config)))
    }
}
