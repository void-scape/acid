use crate::{Config, Process, ops::An};

// pub struct NoteSequence<T> {
//     seq: T,
//     index: usize,
//     len: usize,
//     subdiv: usize,
// }
//
// impl<T> Process for NoteSequence<T>
// where
//     T: Sequence,
// {
//     fn sample(&mut self, _: &Config) -> f32 {
//         todo!()
//     }
// }
//
// pub trait Sequence {
//     fn sample_index(&mut self, config: &Config, index: usize) -> f32;
//     fn len(&self) -> usize;
// }
//
// impl<A, B> Sequence for (A, B)
// where
//     A: Sequence,
//     B: Sequence,
// {
//     fn sample_index(&mut self, config: &Config, index: usize) -> f32 {
//         let (a, b) = self;
//         if index == 0 {
//             a
//         }
//         if index == 1 {
//             b
//         }
//         panic!("index out of bounds in `Sequence::sample_index`");
//     }
//
//     fn len(&self) -> usize {
//         2
//     }
// }

impl<T> NoteExt for T where T: Process {}
pub trait NoteExt: Process + Sized {
    fn cminor(self) -> An<impl Process> {
        slist(
            [
                261.63, 293.66, 311.13, 349.23, 392.00, 415.30, 466.16, 523.25,
            ],
            self,
        )
    }

    fn gminor(self) -> An<impl Process> {
        slist(
            [
                196.00, 220.00, 233.08, 261.63, 293.66, 311.13, 349.23, 392.00,
            ],
            self,
        )
    }

    fn dphrydom(self) -> An<impl Process> {
        slist(
            [
                146.83, 155.56, 185.00, 196.00, 220.00, 233.08, 261.63, 293.66,
            ],
            self,
        )
    }
}

/// Maps `src` [`Process::sample`] to an index into `list`. Assumes samples
/// are in the range `0.0..1.0`.
pub fn slist<const LEN: usize>(list: [f32; LEN], mut src: impl Process) -> An<impl Process> {
    An(move |config: &Config| {
        let index = (src.sample(config) * list.len() as f32) as usize;
        list[index]
    })
}
