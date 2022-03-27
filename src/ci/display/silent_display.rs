use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::CiDisplay;

pub struct SilentDisplay {}
impl CiDisplay for SilentDisplay {
    fn refresh(&mut self, _: &JobProgressTracker, _: usize) {}
    fn finish(&mut self, _: &JobProgressTracker) {}
}
