# Reminder App

A simple, terminal-based reminder application built with Rust.

## Overview

This application allows you to manage your reminders directly from your terminal. It provides a simple and efficient way to add, edit, and delete reminders. The application uses a terminal user interface (TUI) for an interactive experience and stores your reminders in an SQLite database. Additionally, it sends desktop notifications when a reminder is due.

## Features

- **Add, Edit, and Delete Reminders:** Easily manage your reminders with simple keybindings.
- **TUI:** A user-friendly terminal interface for a smooth experience.
- **SQLite Database:** Reminders are persistently stored in an SQLite database.
- **Desktop Notifications:** Get notified when a reminder is due.

## Dependencies

- `chrono`
- `notify-rust`
- `serde_json`
- `serde`
- `ratatui`
- `crossterm`
- `rusqlite`
- `tokio`

## How to Run

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/MamangRust/reminder.git
    cd reminder
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```
3.  **Run the application:**
    ```bash
    ./target/release/reminder
    ```

## Keybindings

### List Mode

- `q`: Quit the application
- `a`: Enter Add mode
- `e`: Enter Edit mode
- `d`: Enter Delete mode
- `Up Arrow`: Navigate up
- `Down Arrow`: Navigate down

### Add/Edit Mode

- `Esc`: Return to List mode
- `Tab`: Move to the next input field
- `BackTab`: Move to the previous input field
- `Enter`: Save the reminder

### Delete Mode

- `y`: Confirm deletion
- `n` or `Esc`: Cancel deletion
