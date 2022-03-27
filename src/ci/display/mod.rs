pub mod sequence_display;
pub mod silent_display;
mod spinner;
mod term_wrapper;

pub enum Mode {
    Silent,
    AllOutput,
}

impl Default for Mode {
    fn default() -> Self {
        Self::AllOutput
    }
}

pub struct CiDisplayConfig {
    pub mode: Mode,
    pub ok: String,
    pub ko: String,
    pub cancelled: String,
    pub show_commands: bool,
    pub spinner: (Vec<String>, usize),
}

impl Default for CiDisplayConfig {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            ok: String::from("✔"),
            ko: String::from("✕"),
            cancelled: String::from("? cancelled"),
            show_commands: true,
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
