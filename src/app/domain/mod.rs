pub mod error;

use error::Error;

use crate::app::domain::Event::Play;
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Consumer {
    /// play allows the domain to start a process in the background
    /// that process should send `Event`s just like self.record to
    /// to signal advencement of the process.
    /// # Errors
    /// when command could not be ran
    fn play(&self, cmd: &str) -> Result<()>;

    /// record allows to future `Event`s to handle.
    /// Event list will expand with the use cases.
    /// # Errors
    /// when an error when queuing item
    fn record(&self, event: Event) -> Result<()>;
}

#[derive(Clone, Debug, Default)]
pub struct State {
    map: HashMap<String, Vec<JobEvent>>,
}

pub struct Job {}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    #[must_use]
    pub fn reserve(&self, name: &str) -> State {
        let mut map = self.map.clone();
        map.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(JobEvent::Reserved);
        State { map }
    }

    fn first_available_job(&self) -> Option<&str> {
        for (name, events) in &self.map {
            if events.is_empty() {
                return Some(name);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub enum JobEvent {
    Reserved,
}

#[derive(Clone, Debug)]
pub enum Event {
    Awaiting,
    Play(String),
}

/// runs the event together with the state
/// # Errors
/// when the consumer itself fails
pub fn run<C: Consumer>(consumer: &C, state: State, event: &Event) -> Result<State> {
    let e = match event {
        Event::Play(_) => state,
        Event::Awaiting => {
            if let Some(job) = state.first_available_job() {
                consumer.record(Play(job.to_string()))?;
                state.reserve(job)
            } else {
                state
            }
        }
    };
    Ok(e)
}
