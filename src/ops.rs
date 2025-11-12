use crate::Process;

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
    fn reset(&mut self) {
        self.0.reset();
    }

    fn sample(&mut self, config: &crate::Config) -> f32 {
        self.0.sample(config)
    }
}

macro_rules! impl_ops {
    ($an:ident, $op:ident) => {
        pub struct $an<Lhs, Rhs> {
            lhs: Lhs,
            rhs: Rhs,
        }

        impl<Lhs, Rhs> core::ops::$an<Rhs> for An<Lhs>
        where
            Lhs: Process,
            Rhs: Process,
        {
            type Output = An<$an<An<Lhs>, Rhs>>;
            fn $op(self, rhs: Rhs) -> Self::Output {
                An($an { lhs: self, rhs })
            }
        }

        impl<Lhs, Rhs> Process for $an<Lhs, Rhs>
        where
            Lhs: Process,
            Rhs: Process,
        {
            fn sample(&mut self, config: &crate::Config) -> f32 {
                use core::ops::$an;
                self.lhs.sample(config).$op(self.rhs.sample(config))
            }
        }
    };
}

impl_ops!(Add, add);
impl_ops!(Sub, sub);
impl_ops!(Mul, mul);
impl_ops!(Div, div);
