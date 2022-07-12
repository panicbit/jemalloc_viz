use anyhow::{anyhow, Result};
use std::fmt::Display;

pub trait JemallocCtlContext<T> {
    fn context<C>(self, context: C) -> Result<T, anyhow::Error>
    where
        C: Display + Send + Sync + 'static;
}

impl<T> JemallocCtlContext<T> for jemalloc_ctl::Result<T> {
    fn context<C>(self, context: C) -> Result<T, anyhow::Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| anyhow!(err))
    }
}
