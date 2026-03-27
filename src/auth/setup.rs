use std::io::{self, stdout};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::config::{Config, CredentialsFile, Profile};
use crate::ui::centered_rect;

const FIELD_PROFILE: usize = 0;
const FIELD_ENDPOINT: usize = 1;
const FIELD_CREDS: usize = 2;

enum AppState {
    Editing,
    Done,
}

struct SetupApp {
    fields: [String; 3],
    focused: usize,
    state: AppState,
    message: Option<String>,
    error: bool,
}

impl SetupApp {
    fn new() -> Self {
        Self {
            fields: [
                "default".to_string(),
                String::new(),
                String::new(),
            ],
            focused: 0,
            state: AppState::Editing,
            message: None,
            error: false,
        }
    }

    fn focus_next(&mut self) {
        self.focused = (self.focused + 1) % 3;
    }

    fn focus_prev(&mut self) {
        self.focused = self.focused.checked_sub(1).unwrap_or(2);
    }

    fn try_save(&mut self) {
        let profile_name = self.fields[FIELD_PROFILE].trim().to_string();
        let api_endpoint = self.fields[FIELD_ENDPOINT].trim().to_string();
        let creds_path_raw = self.fields[FIELD_CREDS].trim().to_string();

        if profile_name.is_empty() || api_endpoint.is_empty() || creds_path_raw.is_empty() {
            self.message = Some("All fields are required.".to_string());
            self.error = true;
            return;
        }

        let expanded = if creds_path_raw.starts_with('~') {
            match dirs::home_dir() {
                Some(home) => {
                    let without_tilde = creds_path_raw.trim_start_matches('~');
                    let stripped = without_tilde.trim_start_matches(['/', '\\']);
                    home.join(stripped).to_string_lossy().into_owned()
                }
                None => creds_path_raw.clone(),
            }
        } else {
            creds_path_raw.clone()
        };

        let creds = match CredentialsFile::load_from_path(&expanded) {
            Ok(c) => c,
            Err(e) => {
                self.message = Some(format!("Error reading credentials: {e}"));
                self.error = true;
                return;
            }
        };

        let profile = Profile {
            api_endpoint,
            api_key: creds.api_key,
            key_id: creds.key_id,
        };

        let mut config = match Config::load() {
            Ok(c) => c,
            Err(e) => {
                self.message = Some(format!("Error loading config: {e}"));
                self.error = true;
                return;
            }
        };

        if let Err(e) = config.add_profile(profile_name.clone(), profile) {
            self.message = Some(format!("Error saving profile: {e}"));
            self.error = true;
            return;
        }

        self.message = Some(format!("✓ Profile '{}' saved.", profile_name));
        self.error = false;
        self.state = AppState::Done;
    }
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = SetupApp::new();

    let labels = ["Profile Name:", "API Endpoint:", "Credentials File:"];
    let placeholders = ["default", "https://commons.example.org", "~/.gen3/credentials.json"];

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let popup = centered_rect(70, 65, area);

            // Outer box
            let outer = Block::default()
                .title(Span::styled(
                    " Gen3 Auth Setup ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            f.render_widget(outer, popup);

            // Inner area with padding
            let inner = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // field 0
                    Constraint::Length(1), // gap
                    Constraint::Length(3), // field 1
                    Constraint::Length(1), // gap
                    Constraint::Length(3), // field 2
                    Constraint::Min(1),    // spacer
                    Constraint::Length(2), // status/instructions
                ])
                .split(popup);

            // Render each input field
            for i in 0..3 {
                let focused = app.focused == i;
                let border_style = if focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let value = &app.fields[i];
                let display_text = if value.is_empty() {
                    Span::styled(
                        placeholders[i],
                        Style::default().fg(Color::DarkGray),
                    )
                } else {
                    Span::styled(value.clone(), Style::default().fg(Color::White))
                };

                // Append cursor block on focused field
                let content = if focused {
                    Line::from(vec![
                        display_text,
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                    ])
                } else {
                    Line::from(display_text)
                };

                let field_widget = Paragraph::new(content).block(
                    Block::default()
                        .title(Span::styled(
                            labels[i],
                            Style::default().add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL)
                        .border_style(border_style),
                );

                let field_area_idx = i * 2; // indices 0, 2, 4
                f.render_widget(field_widget, inner[field_area_idx]);
            }

            // Bottom status line
            let status_area = inner[6];
            match &app.message {
                Some(msg) => {
                    let style = if app.error {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    };
                    f.render_widget(
                        Paragraph::new(Span::styled(msg.clone(), style)),
                        status_area,
                    );
                }
                None => {
                    f.render_widget(
                        Paragraph::new(Span::styled(
                            "Tab/↑↓ to move between fields  Enter to confirm  Esc to cancel",
                            Style::default().fg(Color::DarkGray),
                        )),
                        status_area,
                    );
                }
            }
        })?;

        // After rendering Done state, pause and exit
        if matches!(app.state, AppState::Done) {
            thread::sleep(Duration::from_secs(1));
            break;
        }

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app.focus_prev();
                    } else {
                        app.focus_next();
                    }
                    app.message = None;
                }
                KeyCode::BackTab => {
                    app.focus_prev();
                    app.message = None;
                }
                KeyCode::Up => {
                    app.focus_prev();
                    app.message = None;
                }
                KeyCode::Down => {
                    app.focus_next();
                    app.message = None;
                }
                KeyCode::Enter => {
                    if app.focused == FIELD_CREDS {
                        app.try_save();
                    } else {
                        app.focus_next();
                        app.message = None;
                    }
                }
                KeyCode::Backspace => {
                    app.fields[app.focused].pop();
                    app.message = None;
                }
                KeyCode::Char(c) => {
                    app.fields[app.focused].push(c);
                    app.message = None;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
