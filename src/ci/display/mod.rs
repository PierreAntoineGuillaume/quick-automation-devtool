mod ansi_control_sequence;
pub mod full_final_display;
pub mod interactive_display;
pub mod sequence_display;
pub mod silent_display;
mod spinner;
pub mod summary_display;
mod term_wrapper;

#[derive(Clone, Copy, Debug)]
pub enum RunningDisplay {
    Silent,
    Sequence,
    Summary,
}

#[derive(Clone, Copy, Debug)]
pub enum FinalDisplayMode {
    Full,
    Interactive,
    Silent,
}

impl Default for RunningDisplay {
    fn default() -> Self {
        Self::Sequence
    }
}

impl Default for FinalDisplayMode {
    fn default() -> Self {
        Self::Full
    }
}

pub struct CiDisplayConfig {
    pub running_display: RunningDisplay,
    pub final_display: FinalDisplayMode,
    pub ok: String,
    pub ko: String,
    pub cancelled: String,
    pub spinner: (Vec<String>, usize),
}

impl Default for CiDisplayConfig {
    fn default() -> Self {
        Self {
            running_display: RunningDisplay::default(),
            final_display: FinalDisplayMode::default(),
            ok: String::from("✔"),
            ko: String::from("✕"),
            cancelled: String::from("✕"),
            spinner: (
                vec![
                    String::from(".  "),
                    String::from(".. "),
                    String::from("..."),
                    String::from(".. "),
                    String::from(".  "),
                ],
                80,
            ),
        }
    }
}
