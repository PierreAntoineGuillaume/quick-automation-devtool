use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::ports::FinalCiDisplay;

pub struct InteractiveDisplay<'a> {
    _config: &'a CiDisplayConfig,
}

impl<'a> InteractiveDisplay<'a> {
    pub fn new(config: &'a CiDisplayConfig) -> Self {
        Self { _config: config }
    }
}

impl<'a> FinalCiDisplay for InteractiveDisplay<'a> {
    fn finish(&mut self, _: &JobProgressTracker) {
        println!("ok");
    }
}
