pub trait IntoSignal: Copy {
    fn into_f32(self) -> f32;
}

impl IntoSignal for f32 {
    fn into_f32(self) -> f32 {
        self
    }
}

impl IntoSignal for i32 {
    fn into_f32(self) -> f32 {
        self as f32
    }
}

pub fn clamp(f: f32) -> f32 {
    f.clamp(-1.0, 1.0)
}
