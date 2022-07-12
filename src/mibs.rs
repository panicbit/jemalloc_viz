use anyhow::{anyhow, Context, Result};
use jemalloc_ctl::stats::{active_mib, allocated_mib, mapped_mib, metadata_mib, retained_mib};
use jemalloc_ctl::{epoch, epoch_mib, stats};

use crate::jemalloc_context::JemallocCtlContext;

pub struct MIBs {
    pub epoch: epoch_mib,
    pub active: active_mib,
    pub allocated: allocated_mib,
    pub mapped: mapped_mib,
    pub metadata: metadata_mib,
    pub retained: retained_mib,
}

impl MIBs {
    pub fn advance(&self) -> Result<()> {
        self.epoch.advance().context("failed to advance `epoch`")?;

        Ok(())
    }

    pub fn new() -> Result<Self> {
        Ok(Self {
            epoch: epoch::mib().context("failed to create mib for `epoch`")?,
            active: stats::active::mib().context("failed to create mib for `active`")?,
            allocated: stats::allocated::mib().context("failed to create mib for `allocated`")?,
            mapped: stats::mapped::mib().context("failed to create mib for `mapped`")?,
            metadata: stats::metadata::mib().context("failed to create mib for `metadata`")?,
            retained: stats::retained::mib().context("failed to create mib for `retained`")?,
        })
    }
}
