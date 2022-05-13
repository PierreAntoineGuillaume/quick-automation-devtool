use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::ports::{FinalCiDisplay, UserFacade};

pub struct SilentDisplay {}
impl UserFacade for SilentDisplay {
    fn set_up(&mut self, _: &JobProgressTracker) {}
    fn run(&mut self, _: &JobProgressTracker, _: usize) {}
    fn tear_down(&mut self, _: &JobProgressTracker) {}

    fn display_error(&self, error: String) {
        eprintln!("{}", error)
    }
}

impl FinalCiDisplay for SilentDisplay {
    fn finish(&mut self, _: &JobProgressTracker) {}
}