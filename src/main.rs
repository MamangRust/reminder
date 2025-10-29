mod db;
mod ui;

use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::Database;
use notify_rust::Notification;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::HashSet;
use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
};
use ui::{draw_ui, AppState, Mode};

fn main() -> Result<(), Box<dyn Error>> {
    let db = Database::new("reminders.db")?;
    let reminders = db.get_all_reminders()?;
    let mut app = AppState::new(reminders);

    let notified_ids = Arc::new(Mutex::new(HashSet::new()));
    let notified_ids_clone = Arc::clone(&notified_ids);

    std::thread::spawn(move || {
        notification_worker(notified_ids_clone);
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &db, &mut app, Arc::clone(&notified_ids));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    db: &Database,
    app: &mut AppState,
    _notified_ids: Arc<Mutex<HashSet<i32>>>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    Mode::List => handle_list_input(key, app),
                    Mode::Add => handle_form_input(key, app, db, true),
                    Mode::Edit => handle_form_input(key, app, db, false),
                    Mode::Delete => handle_delete_input(key, app, db),
                }
            }
        }
    }
}

fn handle_list_input(key: KeyEvent, app: &mut AppState) {
    match key.code {
        KeyCode::Char('q') => std::process::exit(0),
        KeyCode::Char('a') => {
            app.mode = Mode::Add;
            app.input.clear();
            app.input_field = 0;
            app.form_fields = [String::new(), String::new(), String::new()];
            app.error_msg = None;
        }
        KeyCode::Char('e') if !app.reminders.is_empty() => {
            app.mode = Mode::Edit;
            app.input.clear();
            app.input_field = 0;
            app.form_fields = [String::new(), String::new(), String::new()];
            app.error_msg = None;
        }
        KeyCode::Char('d') if !app.reminders.is_empty() => {
            app.mode = Mode::Delete;
        }
        KeyCode::Up => app.prev(),
        KeyCode::Down => app.next(),
        _ => {}
    }
}

fn validate_time_format(time: &str) -> bool {
    if time.len() != 5 || !time.contains(':') {
        return false;
    }

    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
        hour < 24 && minute < 60
    } else {
        false
    }
}

fn handle_form_input(key: KeyEvent, app: &mut AppState, db: &Database, is_add: bool) {
    match key.code {
        KeyCode::Char(c) => app.input.push(c),
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Tab => app.next_field(),
        KeyCode::BackTab => app.prev_field(),
        KeyCode::Esc => app.mode = Mode::List,
        KeyCode::Enter => {
            app.form_fields[app.input_field] = app.input.clone();

            if app.form_fields[0].is_empty()
                || app.form_fields[1].is_empty()
                || app.form_fields[2].is_empty()
            {
                app.error_msg = Some("All fields must be filled".to_string());
                return;
            }

            if !validate_time_format(&app.form_fields[2]) {
                app.error_msg = Some("Invalid time format. Use HH:MM (e.g., 06:59)".to_string());
                return;
            }

            let title = app.form_fields[0].clone();
            let description = app.form_fields[1].clone();
            let time = app.form_fields[2].clone();

            if is_add {
                if let Ok(reminder) = db.add_reminder(title, description, time) {
                    app.reminders.push(reminder);
                    app.mode = Mode::List;
                    app.error_msg = None;
                }
            } else if let Some(selected) = app.reminders.get(app.selected_idx) {
                let id = selected.id;
                if db
                    .update_reminder(id, title.clone(), description.clone(), time.clone())
                    .is_ok()
                {
                    if let Some(reminder) = app.reminders.get_mut(app.selected_idx) {
                        reminder.title = title;
                        reminder.description = description;
                        reminder.time = time;
                    }
                    app.mode = Mode::List;
                    app.error_msg = None;
                }
            }
        }
        _ => {}
    }
}

fn handle_delete_input(key: KeyEvent, app: &mut AppState, db: &Database) {
    match key.code {
        KeyCode::Char('y') => {
            if let Some(reminder) = app.reminders.get(app.selected_idx) {
                let id = reminder.id;
                if db.delete_reminder(id).is_ok() {
                    app.reminders.remove(app.selected_idx);
                    if app.selected_idx > 0 && app.selected_idx >= app.reminders.len() {
                        app.selected_idx -= 1;
                    }
                    app.mode = Mode::List;
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => app.mode = Mode::List,
        _ => {}
    }
}

fn notification_worker(notified_ids: Arc<Mutex<HashSet<i32>>>) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(30));

        if let Ok(db) = Database::new("reminders.db") {
            if let Ok(reminders) = db.get_all_reminders() {
                let now = Local::now();
                let current_time = now.format("%H:%M").to_string();

                for reminder in reminders {
                    let mut notified = notified_ids.lock().unwrap();

                    if reminder.time == current_time && !notified.contains(&reminder.id) {
                        match Notification::new()
                            .summary(&reminder.title)
                            .body(&reminder.description)
                            .timeout(5000)
                            .show()
                        {
                            Ok(_) => {
                                notified.insert(reminder.id);
                            }
                            Err(e) => println!("Failed to send notification: {}", e),
                        }
                    }
                }
            }
        }
    }
}
