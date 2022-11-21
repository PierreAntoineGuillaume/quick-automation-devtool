use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::ports::FinalCiDisplay;
use ansi_to_tui::IntoText;
use anyhow::{anyhow, Result};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crossterm::event::{KeyEvent, KeyModifiers, MouseEventKind};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::ci::display::tui::stateful_list::StatefulList;
use crate::ci::display::tui::stateful_text::StatefulText;
use crate::ci::job::{Output, Progress};
use tui::style::Color;
use tui::widgets::{Paragraph, Wrap};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};

pub struct Interactive {
    _config: Option<CiDisplayConfig>,
}

impl Interactive {
    pub fn new(config: &CiDisplayConfig) -> Self {
        Self {
            _config: Some((*config).clone()),
        }
    }
}

impl FinalCiDisplay for Interactive {
    fn finish(&mut self, tracker: &JobProgressTracker) {
        match finish_error(tracker) {
            Ok(()) => {}
            Err(err) => eprintln!("{err}"),
        }
    }
}

fn finish_error(tracker: &JobProgressTracker) -> Result<()> {
    if tracker.states.is_empty() {
        return Err(anyhow!("No jobs to display"));
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);

    let res = run_app(&mut terminal, App::from(tracker), tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

pub enum JobResult {
    Success,
    Failure,
    Unknown,
}

impl From<&Progress> for JobResult {
    fn from(progress: &Progress) -> Self {
        match progress {
            Progress::Skipped | Progress::Terminated(true) => Self::Success,
            Progress::Cancelled | Progress::Terminated(false) => Self::Failure,
            _ => Self::Unknown,
        }
    }
}

struct App<'a> {
    items: StatefulList<(JobResult, String)>,
    tracker: &'a JobProgressTracker,
    result: (bool, StatefulText),
}

impl<'a> App<'a> {
    pub fn previous(&mut self) {
        if self.result.0 {
            self.result.1.previous();
        } else {
            self.items.previous();
            self.result = (false, StatefulText::with_text(self.selected_text()));
        }
    }
    pub fn next(&mut self) {
        if self.result.0 {
            self.result.1.next();
        } else {
            self.items.next();
            self.result = (false, StatefulText::with_text(self.selected_text()));
        }
    }

    pub fn prepare(&mut self) {
        self.result = (false, StatefulText::with_text(self.selected_text()));
    }

    pub fn select(&mut self) {
        self.result = (true, self.result.1.clone());
    }

    pub fn unselect(&mut self) {
        self.result = (false, self.result.1.clone());
    }

    fn selected_text(&self) -> String {
        let selected = self
            .items
            .state
            .selected()
            .or(Some(0))
            .expect("has fallback");

        let collector = match self.tracker.states.iter().nth(selected) {
            Some(item) => item.1,
            None => {
                return String::new();
            }
        };

        let progress_items = collector
            .progresses
            .iter()
            .filter_map(|progres| match progres {
                Progress::Partial(_, Output::Success(out, err) | Output::JobError(out, err)) => {
                    Some(format!("{out}\n{err}"))
                }
                Progress::Partial(_, Output::ProcessError(err)) => Some(err.to_string()),
                Progress::Skipped => Some("skipped".to_string()),
                Progress::Cancelled => Some("cancelled".to_string()),
                Progress::Terminated(true) => Some("success".to_string()),
                Progress::Terminated(false) => Some("failure".to_string()),
                _ => None,
            })
            .collect::<Vec<String>>();
        progress_items.join("\n")
    }
}

impl<'a> From<&'a JobProgressTracker> for App<'a> {
    fn from(tracker: &'a JobProgressTracker) -> Self {
        let mut items = vec![];
        for (name, state) in &tracker.states {
            items.push((JobResult::from(state.last()), name.to_string()));
        }
        Self {
            items: StatefulList::with_items(items),
            tracker,
            result: (false, StatefulText::with_text(String::new())),
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    app.prepare();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if should_exit(&key) {
                    return Ok(());
                }
                match key.code {
                    KeyCode::Right | KeyCode::Enter => app.select(),
                    KeyCode::Left | KeyCode::Backspace => app.unselect(),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
            if let Event::Mouse(event) = event {
                match event.kind {
                    MouseEventKind::ScrollUp => app.previous(),
                    MouseEventKind::ScrollDown => app.next(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

/// In case of sequences, quit :
/// c-d (standard EOF sequence)
/// q-c (standard signal for sigquit)
/// q   (for quit)
/// esc (for escape)
const fn should_exit(e: &KeyEvent) -> bool {
    matches!(
        (e.modifiers, e.code),
        (KeyModifiers::CONTROL, KeyCode::Char('d' | 'c'))
            | (KeyModifiers::NONE, KeyCode::Char('q'))
            | (_, KeyCode::Esc)
    )
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let app_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|(result, item)| {
            ListItem::new(Span::from(item.to_string())).style(Style::default().fg(match result {
                JobResult::Success => Color::Green,
                JobResult::Failure | JobResult::Unknown => Color::Red,
            }))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("jobs"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(match &app.result.0 {
            false => ">  ",
            true => ">> ",
        });

    // We can now render the item list
    f.render_stateful_widget(items, app_chunks[0], &mut app.items.state);

    let text = &app.result.1.text;

    let exp = match text.clone().into_text() {
        Ok(res) => Paragraph::new(res),
        _ => Paragraph::new(text.to_string()),
    };

    f.render_widget(
        exp.wrap(Wrap { trim: false })
            .scroll((app.result.1.scroll, 0))
            .block(Block::default().borders(Borders::ALL).title("result")),
        app_chunks[1],
    );
}
