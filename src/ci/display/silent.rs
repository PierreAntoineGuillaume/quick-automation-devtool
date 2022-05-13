use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::ports::{FinalCiDisplay, UserFacade};

pub struct Display {}
impl UserFacade for Display {
    fn set_up(&mut self, _: &JobProgressTracker) {}
    fn run(&mut self, _: &JobProgressTracker, _: usize) {}
    fn tear_down(&mut self, _: &JobProgressTracker) {}

    fn display_error(&self, error: String) {
        eprintln!("{}", error)
    }
}

impl FinalCiDisplay for Display {
    fn finish(&mut self, _: &JobProgressTracker) {}
}
