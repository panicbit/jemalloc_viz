use anyhow::{Context, Result};
use ringbuffer::{AllocRingBuffer, RingBufferWrite, RingBuffer};

use crate::jemalloc_context::JemallocCtlContext;
use crate::mibs::MIBs;

pub struct Histograms {
    pub active: AllocRingBuffer<f64>,
    pub allocated: AllocRingBuffer<f64>,
    pub mapped: AllocRingBuffer<f64>,
    pub metadata: AllocRingBuffer<f64>,
    pub retained: AllocRingBuffer<f64>,
}

impl Histograms {
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        Ok(Self {
            active: AllocRingBuffer::with_capacity(capacity),
            allocated: AllocRingBuffer::with_capacity(capacity),
            mapped: AllocRingBuffer::with_capacity(capacity),
            metadata: AllocRingBuffer::with_capacity(capacity),
            retained: AllocRingBuffer::with_capacity(capacity),
        })
    }

    pub fn capacity(&self) -> usize {
        self.active.capacity()
    }

    pub fn sample(&mut self, mibs: &MIBs) -> Result<()> {
        self.active.push(mibs.active.read().context("failed to read `active`")? as f64);
        self.allocated.push(mibs.allocated.read().context("failed to read `allocated`")? as f64);
        self.mapped.push(mibs.mapped.read().context("failed to read `mapped`")? as f64);
        self.metadata.push(mibs.metadata.read().context("failed to read `metadata`")? as f64);
        self.retained.push(mibs.retained.read().context("failed to read `retained`")? as f64);

        Ok(())
    }
}
