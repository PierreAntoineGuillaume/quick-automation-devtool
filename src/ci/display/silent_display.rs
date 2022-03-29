use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::RunningCiDisplay;

pub struct SilentDisplay {}
impl RunningCiDisplay for SilentDisplay {
    fn refresh(&mut self, _: &JobProgressTracker, _: usize) {}

    fn clean_up(&mut self) {}
}
