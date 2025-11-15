use crate::{An, Config, F, MonoSrcBound, Process, fmono};

pub fn fadein<const CHANNELS: usize>(duration: f32) -> An<FadeIn<CHANNELS>> {
    An(FadeIn {
        t: 0.0,
        duration: duration as f64,
    })
}

pub struct FadeIn<const CHANNELS: usize> {
    t: f64,
    duration: f64,
}

impl<const CHANNELS: usize> Process for FadeIn<CHANNELS> {
    type Input = F<CHANNELS>;
    type Output = F<CHANNELS>;

    fn reset(&mut self) {
        self.t = 0.0;
    }

    fn sample(&mut self, config: &Config, mut input: Self::Input) -> Self::Output {
        for sample in input.iter_mut() {
            if self.t < self.duration {
                let x = self.t / self.duration;
                let value = ((x * 6.0 - 15.0) * x + 10.0) * x * x * x;
                self.t += config.sample_duration;
                *sample *= value as f32;
            }
        }
        input
    }
}

pub fn lpf<Freq>(cutoff_hz: Freq) -> An<Biquad<Freq, f32, f32, f32>>
where
    Freq: MonoSrcBound,
{
    An(Biquad::new(
        BiquadParams::default(),
        cutoff_hz,
        1.0,
        1.0,
        1.0,
    ))
}

pub struct Biquad<Freq, Q, Env, Depth> {
    params: BiquadParams,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
    freq: Freq,
    q: Q,
    env: Env,
    depth: Depth,
    mfreq: f64,
}

impl<Freq, Q, Env, Depth> Biquad<Freq, Q, Env, Depth> {
    pub fn new(params: BiquadParams, freq: Freq, q: Q, env: Env, depth: Depth) -> Self {
        Biquad {
            params,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            freq,
            q,
            env,
            depth,
            mfreq: 0.0,
        }
    }
}

impl<Freq, Q, Env, Depth> An<Biquad<Freq, Q, Env, Depth>>
where
    Freq: MonoSrcBound,
    Q: MonoSrcBound,
    Env: MonoSrcBound,
    Depth: MonoSrcBound,
{
    pub fn q<Q1>(self, q: Q1) -> An<Biquad<Freq, Q1, Env, Depth>>
    where
        Q1: MonoSrcBound,
    {
        An(Biquad {
            params: self.0.params,
            x1: self.0.x1,
            x2: self.0.x2,
            y1: self.0.y1,
            y2: self.0.y2,
            freq: self.0.freq,
            q,
            env: self.0.env,
            depth: self.0.depth,
            mfreq: self.0.mfreq,
        })
    }

    pub fn env<Env1>(self, env: Env1) -> An<Biquad<Freq, Q, Env1, Depth>>
    where
        Env1: MonoSrcBound,
    {
        An(Biquad {
            params: self.0.params,
            x1: self.0.x1,
            x2: self.0.x2,
            y1: self.0.y1,
            y2: self.0.y2,
            freq: self.0.freq,
            q: self.0.q,
            env,
            depth: self.0.depth,
            mfreq: self.0.mfreq,
        })
    }

    pub fn depth<Depth1>(self, depth: Depth1) -> An<Biquad<Freq, Q, Env, Depth1>>
    where
        Depth1: MonoSrcBound,
    {
        An(Biquad {
            params: self.0.params,
            x1: self.0.x1,
            x2: self.0.x2,
            y1: self.0.y1,
            y2: self.0.y2,
            freq: self.0.freq,
            q: self.0.q,
            env: self.0.env,
            depth,
            mfreq: self.0.mfreq,
        })
    }
}

impl<Freq, Q, Env, Depth> Process for Biquad<Freq, Q, Env, Depth>
where
    Freq: MonoSrcBound,
    Q: MonoSrcBound,
    Env: MonoSrcBound,
    Depth: MonoSrcBound,
{
    type Input = F<1>;
    type Output = F<1>;

    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output {
        let freq = self.freq.filter_mono(config) as f64;
        let q = self.q.filter_mono(config) as f64;
        let env = self.env.filter_mono(config) as f64;
        let depth = self.depth.filter_mono(config) as f64;
        let mfreq = freq * 2f64.powf(depth * env);

        if self.mfreq != mfreq {
            self.params = BiquadParams::lpf(mfreq, q, config.sample_rate);
            self.mfreq = mfreq;
        }

        let x0 = input[0] as f64;
        let y0 = self.params.b0 * x0 + self.params.b1 * self.x1 + self.params.b2 * self.x2
            - self.params.a1 * self.y1
            - self.params.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        fmono(y0 as f32)
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

pub struct Limiter {
    follower: Follower,
    buffer: Vec<f32>,
    index: usize,
}

impl Limiter {
    pub fn new(attack: f32, release: f32, sample_rate: f64) -> Self {
        Self {
            follower: Follower::new(attack, release, sample_rate),
            buffer: (0..440).map(|_| 0.0).collect(),
            index: 0,
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
