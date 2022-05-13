mod ansi_control_sequence;
pub mod exhaustive;
pub mod interactive;
pub mod sequence;
pub mod silent;
mod spinner;
pub mod summary;
mod term_wrapper;
mod tui;

#[derive(Clone, Copy, Debug)]
pub enum Running {
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

impl Default for Running {
    fn default() -> Self {
        Self::Sequence
    }
}

impl Default for FinalDisplayMode {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Clone)]
pub struct CiDisplayConfig {
    pub running_display: Running,
    pub final_display: FinalDisplayMode,
    pub ok: String,
    pub ko: String,
    pub cancelled: String,
    pub spinner: (Vec<String>, usize),
}

impl Default for CiDisplayConfig {
    fn default() -> Self {
        Self {
            running_display: Running::default(),
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
