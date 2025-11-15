#![allow(incomplete_features)]
#![feature(trait_alias)]
#![feature(generic_const_exprs)]

pub mod filter;
pub mod math;
pub mod note;
pub mod osc;
pub mod rng;

pub mod prelude {
    pub use super::filter::*;
    pub use super::note::*;
    pub use super::osc::*;
    pub use super::rng::*;
    pub use super::{An, MonoSrcBound, Process, ResetExt, env, expdecay, fmono, samples};
}

use crate::math::IntoSignal;
use std::marker::PhantomData;

crate::acid!(
    bpm: 140.0,
    kick: kick909(),
    bass: bass() * 0.1,
);

#[allow(clippy::precedence)]
fn kick909() -> An<impl MonoSrcBound> {
    use prelude::*;

    let beat = seq((1, 0, 0, 0)).seg(4);
    let pressure = env(|_, t| fmono(((-t * 16.0).exp() * 125.0) as f32)).res(beat);
    (pressure + 50.0 >> sin() >> fadein(0.005).res(beat))
        * env(|_, t| fmono((-t * 12.0).exp() as f32)).res(beat)
}

#[allow(clippy::precedence)]
fn bass() -> An<impl MonoSrcBound> {
    use prelude::*;

    let notes = rand(28).res(samples(16)).seg(4) >> dphrydom();
    notes / 2 * seq((1, 2, 1, 2, 2)).seg(1)
        >> saw()
        >> lpf(sin_hz(0.5) * 50 + 400)
            .q(12)
            .depth(2)
            .env(expdecay(0.1).res(samples(16)).seg(4))
}

#[macro_export]
macro_rules! mix {
    ($output:expr, $($samples:expr),+) => {
        let output_len = $output.len();
        $(debug_assert!($samples.len() >= output_len);)*
        for i in 0..output_len {
            let mut sum = 0f32;
            $(sum += $samples[i];)+
            $output[i] = sum;
        }
    }
}
#[macro_export]
macro_rules! acid {
    {
        bpm: $bpm:expr,
        $($name:ident: $sound:expr),* $(,)?
    } => {
        // need to send to the audio thread in `glazer::audio_stub`
        unsafe impl Send for Acid {}
        pub struct Acid {
            limiters: Vec<$crate::prelude::Limiter>,
            sounds: Vec<(Box<dyn $crate::Process<Input = (), Output = $crate::F<1>>>, Vec<f32>)>,
        }
        impl Default for Acid {
            fn default() -> Self {
                Self {
                    limiters: Vec::new(),
                    sounds: build_sounds(),
                }
            }
        }
        impl Acid {
            pub fn rebuild_sounds(&mut self) {
                self.sounds = build_sounds();
            }
            pub fn process(&mut self, samples: &mut [f32], sample_rate: u32, channels: usize) {
                let [$($name),*] = &mut self.sounds[..] else { unreachable!() };
                let len = samples.len();
                $(if $name.1.len() < len { $name.1.extend((0..len).map(|_| 0.0)) })*
                let config = $crate::Config {
                    sample_rate: sample_rate as f64,
                    sample_duration: 1.0 / sample_rate as f64,
                    channels,
                    bpm: $bpm,
                    spb: (60.0 / ($bpm)) * sample_rate as f64,
                    bps: 1.0 / ((60.0 / ($bpm)) * sample_rate as f64),
                };
                $(
                    let buf = &mut $name.1[0..len];
                    for frame in buf.chunks_mut(channels) {
                        let sample = $name.0.sample(&config, ());
                        frame.fill(sample.0[0]);
                    }
                )*
                while self.limiters.len() < channels {
                    self.limiters.push(
                        $crate::prelude::Limiter::new(
                            $crate::ms(1.0),
                            $crate::ms(500.0),
                            sample_rate as f64,
                        )
                    );
                }
                $crate::mix!(samples, $($name.1),*);
                for frame in samples.chunks_mut(channels) {
                    for (sample, limiter) in frame.iter_mut().zip(self.limiters.iter_mut()) {
                        *sample = $crate::math::clamp(limiter.limit(*sample));
                    }
                }
            }
        }
        fn build_sounds() -> Vec<(Box<dyn $crate::Process<Input = (), Output = $crate::F<1>>>, Vec<f32>)> {
            vec![$((Box::new($sound), Vec::new())),*]
        }
    };
}

pub struct Config {
    pub sample_rate: f64,
    pub sample_duration: f64,
    pub channels: usize,
    pub bpm: f64,
    pub spb: f64,
    pub bps: f64,
}

pub fn ms(ms: f32) -> f32 {
    ms / 1_000.0
}

pub fn fmono(s: f32) -> F<1> {
    F([s])
}
#[derive(Clone, Copy)]
pub struct F<const CHANNELS: usize>(pub [f32; CHANNELS]);
impl<const CHANNELS: usize> Default for F<CHANNELS> {
    fn default() -> Self {
        Self([0.0; CHANNELS])
    }
}
macro_rules! impl_fops {
    ($trait:ident, $op:ident) => {
        impl<const CHANNELS: usize> core::ops::$trait for F<CHANNELS> {
            type Output = Self;
            fn $op(mut self, rhs: Self) -> Self::Output {
                for i in 0..CHANNELS {
                    self.0[i] = self.0[i].$op(rhs.0[i]);
                }
                self
            }
        }
    };
}
impl_fops!(Add, add);
impl_fops!(Sub, sub);
impl_fops!(Mul, mul);
impl_fops!(Div, div);
impl<const CHANNELS: usize> core::ops::Deref for F<CHANNELS> {
    type Target = [f32; CHANNELS];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const CHANNELS: usize> core::ops::DerefMut for F<CHANNELS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub trait Frame: Default + Copy {
    fn new() -> Self;
    fn channels() -> usize;
    fn as_slice(&self) -> &[f32];
}
impl Frame for () {
    fn new() -> Self {}
    fn channels() -> usize {
        0
    }
    fn as_slice(&self) -> &[f32] {
        &[]
    }
}
impl<const CHANNELS: usize> Frame for F<CHANNELS> {
    fn new() -> Self {
        Self::default()
    }
    fn channels() -> usize {
        CHANNELS
    }
    fn as_slice(&self) -> &[f32] {
        &self.0
    }
}
pub trait Process {
    type Input: Frame;
    type Output: Frame;
    fn reset(&mut self) {}
    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output;
    fn filter_mono(&mut self, config: &Config) -> f32 {
        debug_assert_eq!(Self::Input::channels(), 0);
        debug_assert_eq!(Self::Output::channels(), 1);
        self.sample(config, Self::Input::new()).as_slice()[0]
    }
}
pub trait MonoSrcBound = Process<Input = (), Output = F<1>>;
impl<T> Process for T
where
    T: IntoSignal,
{
    type Input = ();
    type Output = F<1>;
    fn sample(&mut self, _: &Config, _: Self::Input) -> Self::Output {
        fmono(self.into_f32())
    }
}

pub fn src<Func, const CHANNELS: usize>(f: Func) -> An<SrcFunc<CHANNELS, Func>>
where
    SrcFunc<CHANNELS, Func>: Process<Input = (), Output = F<CHANNELS>>,
{
    An(SrcFunc(f, PhantomData))
}

#[derive(Clone, Copy)]
pub struct SrcFunc<const N: usize, Func>(Func, PhantomData<F<N>>);
pub type MonoSrc<F> = SrcFunc<1, F>;
pub type StereoSrc<F> = SrcFunc<2, F>;

impl<const N: usize, Func> Process for SrcFunc<N, Func>
where
    Func: FnMut(&Config) -> F<N>,
{
    type Input = ();
    type Output = F<N>;
    fn sample(&mut self, config: &Config, _: Self::Input) -> Self::Output {
        (self.0)(config)
    }
}

pub fn process<Input, Output, Func>(f: Func) -> An<ProcessFunc<Input, Output, Func>>
where
    ProcessFunc<Input, Output, Func>: Process<Input = Input, Output = Output>,
    Func: FnMut(&Config, Input) -> Output,
{
    An(ProcessFunc(f, PhantomData))
}

#[derive(Clone, Copy)]
pub struct ProcessFunc<Input, Output, Func>(Func, PhantomData<(Input, Output)>);
pub type MonoProcess<Func> = ProcessFunc<F<1>, F<1>, Func>;
pub type StereoProcess<Func> = ProcessFunc<F<2>, F<2>, Func>;

impl<Input, Output, Func> Process for ProcessFunc<Input, Output, Func>
where
    Func: FnMut(&Config, Input) -> Output,
    Input: Frame,
    Output: Frame,
{
    type Input = Input;
    type Output = Output;
    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output {
        (self.0)(config, input)
    }
}

pub fn env<Func, const CHANNELS: usize>(f: Func) -> An<EnvFunc<CHANNELS, Func>>
where
    EnvFunc<CHANNELS, Func>: Process<Input = (), Output = F<CHANNELS>>,
    Func: FnMut(&Config, f64) -> F<CHANNELS>,
{
    An(EnvFunc(f, 0.0, PhantomData))
}

#[derive(Clone, Copy)]
pub struct EnvFunc<const N: usize, Func>(Func, f64, PhantomData<F<N>>);
pub type MonoEnv<F> = EnvFunc<1, F>;
pub type StereoEnv<F> = EnvFunc<2, F>;

impl<const N: usize, Func> Process for EnvFunc<N, Func>
where
    Func: FnMut(&Config, f64) -> F<N>,
{
    type Input = ();
    type Output = F<N>;
    fn reset(&mut self) {
        self.1 = 0.0;
    }
    fn sample(&mut self, config: &Config, _: Self::Input) -> Self::Output {
        let sample = (self.0)(config, self.1);
        self.1 += config.sample_duration;
        sample
    }
}

pub fn expdecay(duration: f64) -> An<MonoEnv<impl FnMut(&Config, f64) -> F<1>>> {
    env(move |_, t| fmono((-5.0 * t / duration).exp() as f32))
}

#[derive(Clone, Copy)]
pub struct An<T: Process>(pub T);
impl<T: Process> core::ops::Deref for An<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Process> core::ops::DerefMut for An<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> Process for An<T>
where
    T: Process,
{
    type Input = T::Input;
    type Output = T::Output;
    fn reset(&mut self) {
        self.0.reset();
    }
    fn sample(&mut self, config: &crate::Config, input: Self::Input) -> Self::Output {
        self.0.sample(config, input)
    }
}

macro_rules! impl_ops {
    ($an:ident, $op:ident) => {
        pub struct $an<X, Y> {
            x: X,
            y: Y,
        }

        impl<X, Y> core::ops::$an<Y> for An<X>
        where
            X: MonoSrcBound,
            Y: MonoSrcBound,
            $an<X, Y>: Process,
        {
            type Output = An<$an<X, Y>>;
            fn $op(self, rhs: Y) -> Self::Output {
                An($an { x: self.0, y: rhs })
            }
        }

        impl<X, Y> Process for $an<X, Y>
        where
            X: Process<Input = (), Output = Y::Output>,
            Y: Process<Input = ()>,
            Y::Output: core::ops::$an<Y::Output>,
            <X::Output as core::ops::$an<X::Output>>::Output: Frame,
        {
            type Input = Y::Input;
            type Output = <X::Output as core::ops::$an<X::Output>>::Output;
            fn sample(&mut self, config: &Config, _: Self::Input) -> Self::Output {
                use core::ops::$an;
                let x = self.x.sample(config, ());
                let y = self.y.sample(config, ());
                x.$op(y)
            }
        }
    };
}

impl_ops!(Add, add);
impl_ops!(Sub, sub);
impl_ops!(Mul, mul);
impl_ops!(Div, div);

pub struct Pipe<X, Y> {
    x: X,
    y: Y,
}
impl<X, Y> core::ops::Shr<Y> for An<X>
where
    X: Process<Output = Y::Input>,
    Y: Process,
{
    type Output = An<Pipe<X, Y>>;
    fn shr(self, rhs: Y) -> Self::Output {
        An(Pipe { x: self.0, y: rhs })
    }
}
impl<X, Y> Process for Pipe<X, Y>
where
    X: Process<Output = Y::Input>,
    Y: Process,
{
    type Input = X::Input;
    type Output = Y::Output;

    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output {
        let yinput = self.x.sample(config, input);
        self.y.sample(config, yinput)
    }
}

pub struct Stack<X, Y> {
    x: X,
    y: Y,
}
impl<const X_OUT: usize, const Y_OUT: usize, X, Y> core::ops::BitOr<Y> for An<X>
where
    X: Process<Input = Y::Input, Output = F<X_OUT>>,
    Y: Process<Output = F<Y_OUT>>,
    [(); X_OUT + Y_OUT]:,
{
    type Output = An<Stack<X, Y>>;
    fn bitor(self, rhs: Y) -> Self::Output {
        An(Stack { x: self.0, y: rhs })
    }
}
impl<const X_OUT: usize, const Y_OUT: usize, X, Y> Process for Stack<X, Y>
where
    X: Process<Input = Y::Input, Output = F<X_OUT>>,
    Y: Process<Output = F<Y_OUT>>,
    [(); X_OUT + Y_OUT]:,
{
    type Input = X::Input;
    type Output = F<{ X_OUT + Y_OUT }>;

    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output {
        let x_out = self.x.sample(config, input);
        let y_out = self.y.sample(config, input);
        let mut out = F([0.0; X_OUT + Y_OUT]);
        out[..X_OUT].copy_from_slice(x_out.as_slice());
        out[X_OUT..].copy_from_slice(y_out.as_slice());
        out
    }
}

pub fn pass<const CHANNELS: usize>(
    channel: usize,
) -> An<ProcessFunc<F<CHANNELS>, F<1>, impl FnMut(&Config, F<CHANNELS>) -> F<1>>> {
    process(move |_, input: F<CHANNELS>| fmono(input[channel]))
}

pub fn c(c: f32) -> An<C<1>> {
    An(C(F([c])))
}
pub struct C<const N: usize>(F<N>);
impl<const N: usize> Process for C<N> {
    type Input = ();
    type Output = F<N>;
    fn sample(&mut self, _: &Config, _: Self::Input) -> Self::Output {
        self.0
    }
}

crate::impl_wrapper_ext! {
    pub trait ResetExt {
        fn res<Trig>(self, trig: Trig) -> An<Reset<Self, Trig>>
        where
            Trig: MonoSrcBound
        {
            An(Reset {
                src:self,
                trig,
                triggered: false,
            })
        }
    }
}

pub struct Reset<Src, Trig> {
    src: Src,
    trig: Trig,
    triggered: bool,
}
impl<Src, Trig> Process for Reset<Src, Trig>
where
    Src: Process,
    Trig: Process<Input = (), Output = F<1>>,
{
    type Input = Src::Input;
    type Output = Src::Output;
    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output {
        let trigger = self.trig.sample(config, ())[0];
        if !self.triggered && trigger > 0.0 {
            self.triggered = true;
            self.src.reset();
        } else if self.triggered && trigger <= 0.0 {
            self.triggered = false;
        }
        self.src.sample(config, input)
    }
}

pub fn samples(samples: usize) -> An<impl MonoSrcBound> {
    count(samples).map(move |c| fmono((c[0] as usize == samples) as u32 as f32))
}

pub fn count(n: usize) -> An<MonoSrc<impl FnMut(&Config) -> F<1>>> {
    let mut x = 0;
    src(move |_: &Config| {
        let xx = x;
        x += 1;
        if x > n {
            x = 0;
        }
        fmono(xx as f32)
    })
}

crate::impl_wrapper_ext! {
    pub trait CombExt {
        #[allow(clippy::type_complexity)]
        fn map<Func>(mut self, mut f: Func) ->
            An<ProcessFunc<Self::Input, Self::Output, impl FnMut(&Config, Self::Input) -> Self::Output>>
        where
            Func: FnMut(Self::Output) -> Self::Output,
        {
            process(move |config: &Config, input: Self::Input| f(self.sample(config, input)))
        }
    }
}

#[macro_export]
macro_rules! impl_wrapper_ext {
    {
        pub trait $trait:ident {
            $($tokens:tt)*
        }
    } => {
        impl<T> $trait for T where T: $crate::Process {}
        pub trait $trait: Process + Sized {
            $($tokens)*
        }
    };
}
