use crate::{An, Config, F, MonoProcess, MonoSrc, Process, fmono, process, src};

pub fn sample(mut seq: impl Sequence) -> An<MonoProcess<impl FnMut(&Config, F<1>) -> F<1>>> {
    let len = seq.len() as f32;
    process(move |_, f: F<1>| {
        let index = (f[0].clamp(0.0, 1.0) * len) as usize;
        let sample = seq.sample_index(index);
        fmono(sample)
    })
}

pub fn cminor() -> An<MonoProcess<impl FnMut(&Config, F<1>) -> F<1>>> {
    sample((261.63, 293.66, 311.13, 349.23, 392, 415.30, 466.16, 523.25))
}

pub fn gminor() -> An<MonoProcess<impl FnMut(&Config, F<1>) -> F<1>>> {
    sample((196, 220, 233.08, 261.63, 293.66, 311.13, 349.23, 392))
}

pub fn dphrydom() -> An<MonoProcess<impl FnMut(&Config, F<1>) -> F<1>>> {
    sample((146.83, 155.56, 185, 196, 220, 233.08, 261.63, 293.66))
}

pub fn seq(mut seq: impl Sequence) -> An<MonoSrc<impl FnMut(&Config) -> F<1> + Copy>> {
    let mut index = 0;
    let len = seq.len();
    src(move |_| {
        let sample = seq.sample_index(index);
        index = (index + 1) % len;
        fmono(sample)
    })
}

crate::impl_wrapper_ext! {
    pub trait SegExt {
        fn seg(self, seg: usize) -> An<Seg<Self>> {
            An(Seg::new(self, seg))
        }
    }
}

#[derive(Clone, Copy)]
pub struct Seg<Src: Process> {
    src: Src,
    seg: f64,
    t: f64,
    retained: Src::Output,
    init: bool,
}

impl<Src: Process> Seg<Src> {
    pub fn new(src: Src, seg: usize) -> Self {
        Self {
            src,
            seg: seg as f64,
            t: 0.0,
            retained: Src::Output::default(),
            init: false,
        }
    }
}

impl<Src> Process for Seg<Src>
where
    Src: Process,
{
    type Input = Src::Input;
    type Output = Src::Output;

    fn sample(&mut self, config: &Config, input: Self::Input) -> Self::Output {
        let duration = config.spb / self.seg;
        if !self.init || self.t >= duration {
            self.init = true;
            self.t -= duration;
            self.retained = self.src.sample(config, input);
        }
        self.t += 1.0;
        self.retained
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait Sequence: Copy {
    fn sample_index(&mut self, index: usize) -> f32;
    fn len(&self) -> usize;
}

macro_rules! impl_seq {
    ($($param:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($param: $crate::math::IntoSignal,)*> Sequence for ($($param),*) {
            fn sample_index(&mut self, index: usize) -> f32 {
                let ($($param),*) = self;
                let mut i = 0;
                $(
                    if i == index { return $param.into_f32(); }
                    i += 1;
                )*
                panic!("index {} out of bounds for `Sequence::sample_index`", index);
            }
            fn len(&self) -> usize {
                let ($($param),*) = self;
                let mut len = 0;
                $(
                    _ = $param;
                    len += 1;
                )*
                len
            }
        }
    };
}

impl_seq!(F1, F2);
impl_seq!(F1, F2, F3);
impl_seq!(F1, F2, F3, F4);
impl_seq!(F1, F2, F3, F4, F5);
impl_seq!(F1, F2, F3, F4, F5, F6);
impl_seq!(F1, F2, F3, F4, F5, F6, F7);
impl_seq!(F1, F2, F3, F4, F5, F6, F7, F8);
