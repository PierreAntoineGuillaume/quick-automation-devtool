#[allow(clippy::wildcard_imports)]
use crate::app::domain::*;

use std::sync::mpsc::Sender;

pub struct Fake {
    pub stream: Sender<Event>,
}

impl Consumer for Fake {
    fn play(&self, _cmd: &str) -> Result<()> {
        Ok(())
    }

    fn record(&self, event: Event) -> Result<()> {
        self.stream.send(event).map_err(|e| error::err(&e))
    }
}
