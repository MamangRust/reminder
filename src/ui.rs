use ratatui::{
    Frame, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph}
};
use crate::db::Reminder;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    List,
    Add,
    Edit,
    Delete,
}

pub struct AppState {
    pub mode: Mode,
    pub reminders: Vec<Reminder>,
    pub selected_idx: usize,
    pub input: String,
    pub input_field: usize,
    pub form_fields: [String; 3],
    pub error_msg: Option<String>,
}

impl AppState {
    pub fn new(reminders: Vec<Reminder>) -> Self {
        AppState {
            mode: Mode::List,
            reminders,
            selected_idx: 0,
            input: String::new(),
            input_field: 0,
            form_fields: [String::new(), String::new(), String::new()],
            error_msg: None,
        }
    }

    pub fn next(&mut self) {
        if self.mode == Mode::List && !self.reminders.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.reminders.len();
        }
    }

    pub fn prev(&mut self) {
        if self.mode == Mode::List && !self.reminders.is_empty() {
            self.selected_idx = if self.selected_idx == 0 {
                self.reminders.len() - 1
            } else {
                self.selected_idx - 1
            };
        }
    }

    pub fn next_field(&mut self) {
        if self.mode == Mode::Add || self.mode == Mode::Edit {
            self.form_fields[self.input_field] = self.input.clone();
            self.input_field = (self.input_field + 1) % 3;
            self.input = self.form_fields[self.input_field].clone();
        }
    }

    pub fn prev_field(&mut self) {
        if self.mode == Mode::Add || self.mode == Mode::Edit {
            self.form_fields[self.input_field] = self.input.clone();
            self.input_field = if self.input_field == 0 { 2 } else { self.input_field - 1 };
            self.input = self.form_fields[self.input_field].clone();
        }
    }
}

pub fn draw_ui(f: &mut Frame, app: &AppState) {
    match app.mode {
        Mode::List => draw_list(f, app),
        Mode::Add => draw_add_form(f, app),
        Mode::Edit => draw_edit_form(f, app),
        Mode::Delete => draw_delete_confirm(f, app),
    }
}

fn draw_list(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(4)])
        .split(f.size());

    let items: Vec<ListItem> = app
        .reminders
        .iter()
        .enumerate()
        .map(|(i, reminder)| {
            let style = if i == app.selected_idx {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!("[{}] {} - {}", reminder.time, reminder.title, reminder.description);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("📝 Reminders"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, chunks[0]);

    let help_text = vec![
        Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate | "),
            Span::styled("a", Style::default().fg(Color::Green)),
            Span::raw(" Add | "),
            Span::styled("e", Style::default().fg(Color::Blue)),
            Span::raw(" Edit | "),
            Span::styled("d", Style::default().fg(Color::Red)),
            Span::raw(" Delete | "),
            Span::styled("q", Style::default().fg(Color::Magenta)),
            Span::raw(" Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center);

    f.render_widget(help, chunks[1]);
}

fn draw_add_form(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let fields = [
        ("Title", "Enter title"),
        ("Description", "Enter description"),
        ("Time (HH:MM)", "Enter time in HH:MM format"),
    ];

    for (i, (label, hint)) in fields.iter().enumerate() {
        let style = if i == app.input_field {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = if i == app.input_field {
            format!("{}> {}", label, app.input)
        } else {
            format!("{}: (empty)", label)
        };

        let widget = Paragraph::new(title)
            .block(Block::default().borders(Borders::ALL).title(*hint))
            .style(style);

        f.render_widget(widget, form_chunks[i]);
    }

    let help = Paragraph::new("Tab: Next field | Shift+Tab: Prev field | Enter: Save | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(help, form_chunks[3]);

    if let Some(err) = &app.error_msg {
        let error = Paragraph::new(err.clone())
            .block(Block::default().borders(Borders::ALL).title("Error"))
            .style(Style::default().fg(Color::Red));
        f.render_widget(error, chunks[1]);
    }
}

fn draw_edit_form(f: &mut Frame, app: &AppState) {
    draw_add_form(f, app);
}

fn draw_delete_confirm(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    if let Some(reminder) = app.reminders.get(app.selected_idx) {
        let msg = format!("Delete reminder: '{}'?", reminder.title);
        let confirm = Paragraph::new(vec![
            Line::from(msg),
            Line::from(""),
            Line::from(vec![
                Span::styled("y", Style::default().fg(Color::Green)),
                Span::raw(" - Yes | "),
                Span::styled("n", Style::default().fg(Color::Red)),
                Span::raw(" - No"),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Confirm Delete"))
        .alignment(Alignment::Center);

        f.render_widget(confirm, chunks[0]);
    }
}