use crate::apu::{
    device::{Audio, Sample},
    ApuInner,
};
use std::{
    cell::Cell,
    sync::{Arc, Mutex, MutexGuard},
};

/// A mutex before the APU samples.
pub struct SamplesMutex<D: Audio> {
    inner: Arc<Mutex<ApuInner<D>>>,
    buf: Arc<Cell<Option<SampleBuffer<D>>>>,
}

// SamplesMutex is not send by default because of the usage of Cell. Safety is
// handled by self.inner
unsafe impl<D: Audio> Send for SamplesMutex<D> {}

impl<D: Audio> SamplesMutex<D> {
    pub(super) fn new(inner: &Arc<Mutex<ApuInner<D>>>) -> Self {
        SamplesMutex {
            inner: Arc::clone(inner),
            buf: Arc::new(Cell::new(None)),
        }
    }

    pub fn lock<'a>(&'a self) -> impl Iterator<Item = D::Sample> + 'a {
        Samples {
            inner: self.inner.lock().expect("Error locking APU"),
            buf: Arc::clone(&self.buf),
        }
    }
}

enum SampleBuffer<D: Audio> {
    Two([D::Sample; 2]),
    One([D::Sample; 1]),
}

/// Iterator of samples produced by the APU.
struct Samples<'a, D: Audio> {
    inner: MutexGuard<'a, ApuInner<D>>,
    buf: Arc<Cell<Option<SampleBuffer<D>>>>,
}

impl<D: Audio> Samples<'_, D> {
    // Loads the next sample into the buffer
    fn load(&mut self) {
        let apu = &mut self.inner;

        // sample new voices
        let ch0 = apu.ch0;
        let ch1 = apu.ch1;
        let ch2 = apu.ch2;
        let ch3 = apu.ch3;

        // audio mixing
        let mut so: [f64; 2] = [0.0; 2];
        let mut count: [u32; 2] = [0; 2];

        let nr51 = apu.nr51;
        for (ch, sample) in [ch0, ch1, ch2, ch3].iter().copied().enumerate() {
            let so1_bit = 1 << (ch as u8);
            let so2_bit = 1 << (4 + ch as u8);
            let sample = if let Some(sample) = sample {
                sample
            } else {
                0.0
            };
            if nr51 & so1_bit != 0 {
                so[0] += sample;
                count[0] += 1;
            }
            if nr51 & so2_bit != 0 {
                so[1] += sample;
                count[1] += 1;
            }
        }

        if count[0] > 0 {
            so[0] /= count[0] as f64;
        }
        if count[1] > 0 {
            so[1] /= count[1] as f64;
        }

        let max: f64 = D::Sample::max().as_f64();
        let min: f64 = D::Sample::min().as_f64();
        let l = clamp(so[0] * 0.5 + 0.5, 0.0, 1.0);
        let l = min * (1.0 - l) + max * l;
        let r = clamp(so[1] * 0.5 + 0.5, 0.0, 1.0);
        let r = min * (1.0 - r) + max * r;

        self.buf.set(Some(if D::mono() {
            let mix = D::Sample::from_f64((l + r) / 2.0);
            SampleBuffer::One([mix])
        } else {
            let l = D::Sample::from_f64(l);
            let r = D::Sample::from_f64(r);
            SampleBuffer::Two([l, r])
        }));
    }
}

impl<D: Audio> Iterator for Samples<'_, D> {
    type Item = D::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.buf.take() {
                Some(SampleBuffer::Two([ch0, ch1])) => {
                    self.buf.set(Some(SampleBuffer::One([ch1])));
                    return Some(ch0);
                }
                Some(SampleBuffer::One([ch])) => {
                    self.buf.set(None);
                    return Some(ch);
                }
                None => self.load(),
            }
        }
    }
}

fn clamp(n: f64, min: f64, max: f64) -> f64 {
    if n > max {
        max
    } else if n < min {
        min
    } else {
        n
    }
}
