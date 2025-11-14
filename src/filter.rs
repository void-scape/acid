use crate::{Config, Process, init, ops::An};

impl<T> FilterExt for T where T: Process {}
pub trait FilterExt: Process + Sized {
    fn limit(self, attack: f32, release: f32) -> An<impl Process> {
        An(init(move |c| {
            Limiter::new(self, attack, release, c.sample_rate)
        }))
    }

    fn lpf<Freq>(self, cutoff_hz: Freq) -> An<Biquad<Self, Freq, f32, f32>>
    where
        Freq: Process,
    {
        An(Biquad {
            params: BiquadParams::default(),
            freq: cutoff_hz,
            q: 1.0,
            src: self,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            env: 1.0,
            depth: 1.0,
            mfreq: 0.0,
        })
    }

    fn fadein(self, duration: f32) -> An<FadeIn<Self>> {
        An(FadeIn {
            t: 0.0,
            duration: duration as f64,
            src: self,
        })
    }
}

pub struct Biquad<Src, Freq, Q, Env> {
    params: BiquadParams,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
    src: Src,
    freq: Freq,
    q: Q,
    env: Env,
    depth: f64,
    mfreq: f64,
}

impl<Src, Freq, Q, Env> An<Biquad<Src, Freq, Q, Env>>
where
    Src: Process,
    Freq: Process,
    Q: Process,
    Env: Process,
{
    pub fn q<Q1>(self, q: Q1) -> An<Biquad<Src, Freq, Q1, Env>>
    where
        Q1: Process,
    {
        An(Biquad {
            params: self.0.params,
            x1: self.0.x1,
            x2: self.0.x2,
            y1: self.0.y1,
            y2: self.0.y2,
            src: self.0.src,
            freq: self.0.freq,
            q,
            env: self.0.env,
            depth: self.0.depth,
            mfreq: self.0.mfreq,
        })
    }

    pub fn env<Env1>(self, env: Env1) -> An<Biquad<Src, Freq, Q, Env1>>
    where
        Env1: Process,
    {
        An(Biquad {
            params: self.0.params,
            x1: self.0.x1,
            x2: self.0.x2,
            y1: self.0.y1,
            y2: self.0.y2,
            src: self.0.src,
            freq: self.0.freq,
            q: self.0.q,
            env,
            depth: self.0.depth,
            mfreq: self.0.mfreq,
        })
    }

    pub fn depth(mut self, depth: f64) -> Self {
        self.0.depth = depth;
        self
    }
}

/// Implementation based on these resources:
/// - https://github.com/SamiPerttu/fundsp/blob/master/src/biquad.rs
/// - https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
#[derive(Default)]
pub struct BiquadParams {
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}

impl BiquadParams {
    pub fn lpf(cutoff_hz: f64, q: f64, sample_rate: f64) -> Self {
        let omega = core::f64::consts::TAU * cutoff_hz / sample_rate;
        let (osin, ocos) = omega.sin_cos();
        let alpha = osin / (2.0 * q);
        let a0 = 1.0 + alpha;
        let ocosm1 = 1.0 - ocos;
        let b0 = (ocosm1 / 2.0) / a0;
        let b1 = ocosm1 / a0;
        let b2 = b0;
        let a1 = (-2.0 * ocos) / a0;
        let a2 = (1.0 - alpha) / a0;
        Self { a1, a2, b0, b1, b2 }
    }
}

impl<Src, Freq, Q, Env> Process for Biquad<Src, Freq, Q, Env>
where
    Src: Process,
    Freq: Process,
    Q: Process,
    Env: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let freq = self.freq.sample(config) as f64;
        let q = self.q.sample(config) as f64;
        let env = self.env.sample(config) as f64;
        let mfreq = freq * 2f64.powf(self.depth * env);

        if self.mfreq != mfreq {
            self.params = BiquadParams::lpf(mfreq, q, config.sample_rate);
            self.mfreq = mfreq;
        }

        let x0 = self.src.sample(config) as f64;
        let y0 = self.params.b0 * x0 + self.params.b1 * self.x1 + self.params.b2 * self.x2
            - self.params.a1 * self.y1
            - self.params.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        y0 as f32
    }
}

pub struct Limiter<Src> {
    follower: Follower,
    buffer: Vec<f32>,
    index: usize,
    src: Src,
}

impl<Src> Limiter<Src> {
    pub fn new(src: Src, attack: f32, release: f32, sample_rate: f64) -> Self {
        Self {
            follower: Follower::new(attack, release, sample_rate),
            buffer: (0..440).map(|_| 0.0).collect(),
            index: 0,
            src,
        }
    }

    pub fn limit(&mut self, sample: f32) -> f32 {
        let output = self.buffer[self.index];
        self.buffer[self.index] = sample;
        let sample = output;
        self.index = (self.index + 1) % self.buffer.len();
        let peak = self
            .buffer
            .iter()
            .map(|&s| s.abs() as f64)
            .fold(0.0, f64::max);
        let limit = self.follower.filter(1f64.max(peak * 1.1));
        sample / limit
    }
}

impl<Src> Process for Limiter<Src>
where
    Src: Process,
{
    fn sample(&mut self, config: &Config) -> f32 {
        let sample = self.src.sample(config);
        self.limit(sample)
    }
}

struct Follower {
    env: f64,
    att: f64,
    rel: f64,
}

impl Follower {
    fn new(attack: f32, release: f32, sample_rate: f64) -> Self {
        Self {
            env: 0.0,
            att: 0.01f64.powf(1.0 / (attack as f64 * sample_rate * 0.001)),
            rel: 0.01f64.powf(1.0 / (release as f64 * sample_rate * 0.001)),
        }
    }

    fn filter(&mut self, sample: f64) -> f32 {
        let abs = sample.abs();
        if abs > self.env {
            self.env = self.att * (self.env - abs) + abs;
        } else {
            self.env = self.rel * (self.env - abs) + abs;
        }
        self.env as f32
    }
}

pub struct FadeIn<Src> {
    t: f64,
    duration: f64,
    src: Src,
}

impl<Src> Process for FadeIn<Src>
where
    Src: Process,
{
    fn reset(&mut self) {
        self.t = 0.0;
    }

    fn sample(&mut self, config: &Config) -> f32 {
        let sample = self.src.sample(config);
        if self.t < self.duration {
            let x = self.t / self.duration;
            let value = ((x * 6.0 - 15.0) * x + 10.0) * x * x * x;
            self.t += config.sample_duration;
            sample * value as f32
        } else {
            sample
        }
    }
}
