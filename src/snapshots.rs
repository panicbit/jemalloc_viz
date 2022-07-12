use std::cmp::Ordering;

use ringbuffer::{RingBufferExt, AllocRingBuffer};

use crate::histograms::Histograms;

pub struct Snapshots {
    // TODO: turn these back into Vec<size_t> (see MIBs)
    pub active: Vec<(f64, f64)>,
    pub allocated: Vec<(f64, f64)>,
    pub mapped: Vec<(f64, f64)>,
    pub metadata: Vec<(f64, f64)>,
    pub retained: Vec<(f64, f64)>,
}

impl Snapshots {
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            allocated: Vec::new(),
            mapped: Vec::new(),
            metadata: Vec::new(),
            retained: Vec::new(),
        }
    }

    pub fn max(&self) -> usize {
        self.active.iter()
            .chain(&self.allocated)
            .chain(&self.mapped)
            .chain(&self.metadata)
            .chain(&self.retained)
            .map(|(_, n)| *n as usize)
            .max()
            .unwrap_or(0)
    }

    pub fn snapshot_histograms(&mut self, histograms: &Histograms) {
        Self::snapshot(&mut self.active, &histograms.active);
        Self::snapshot(&mut self.allocated, &histograms.allocated);
        Self::snapshot(&mut self.mapped, &histograms.mapped);
        Self::snapshot(&mut self.metadata, &histograms.metadata);
        Self::snapshot(&mut self.retained, &histograms.retained);
    }

    fn snapshot(snapshot: &mut Vec<(f64, f64)>, histogram: &AllocRingBuffer<f64>) {
        snapshot.clear();

        let histogram = histogram.iter()
            .enumerate()
            .map(|(i, value)| (i as f64, *value));

        snapshot.extend(histogram);
    }
}
