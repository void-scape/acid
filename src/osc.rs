use crate::{Config, F, MonoProcess, MonoSrcBound, An, c, fmono, process};

pub fn sin() -> An<MonoProcess<impl FnMut(&Config, F<1>) -> F<1>>> {
    let mut phase = 0f64;
    process(move |config, freq: F<1>| {
        let freq = freq[0];
        let p = phase as f32;
        phase += freq as f64 * config.sample_duration;
        phase = phase.fract();
        fmono((p * core::f32::consts::TAU).sin())
    })
}

pub fn sin_hz(hz: f32) -> An<impl MonoSrcBound> {
    c(hz) >> sin()
}

pub fn saw() -> An<MonoProcess<impl FnMut(&Config, F<1>) -> F<1>>> {
    let mut phase = 0f64;
    process(move |config, freq: F<1>| {
        let freq = freq[0];
        let p = phase as f32;
        phase += freq as f64 * config.sample_duration;
        phase = phase.fract();
        fmono((p % 1.0) * 2.0 - 1.0)
    })
}

pub fn saw_hz(hz: f32) -> An<impl MonoSrcBound> {
    c(hz) >> saw()
}
