use crate::ci::job::JobOutput;

#[derive(Debug, PartialEq)]
pub enum Progress {
    Available,
    Started,
    Partial(String, JobOutput),
    Terminated(bool),
}

impl Progress {
    pub fn failed(&self) -> bool {
        matches!(
            self,
            Progress::Partial(_, JobOutput::JobError(_, _))
                | Progress::Partial(_, JobOutput::ProcessError(_))
                | Progress::Terminated(false)
        )
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Progress::Available)
    }

    pub fn is_pending(&self) -> bool {
        !matches!(*self, Progress::Terminated(_))
    }
}
