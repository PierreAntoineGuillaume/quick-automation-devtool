mod full_final_display;
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
    pub spinner: (Vec<String>, usize),
}

impl Default for CiDisplayConfig {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            ok: String::from("✔"),
            ko: String::from("✕"),
            cancelled: String::from("? cancelled"),
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
