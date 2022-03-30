use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::RunningCiDisplay;

pub struct SilentDisplay {}
impl RunningCiDisplay for SilentDisplay {
    fn set_up(&mut self, _: &JobProgressTracker) {}
    fn run(&mut self, _: &JobProgressTracker, _: usize) {}
    fn tear_down(&mut self, _: &JobProgressTracker) {}
}
