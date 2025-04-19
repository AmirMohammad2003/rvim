use std::{io::stdout, process::exit};

use crossterm::{execute, terminal};

pub fn graceful_exit(code: i32) -> Result<(), std::io::Error> {
    execute!(
        stdout(),
        terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;
    terminal::disable_raw_mode()?;
    exit(code)
}
