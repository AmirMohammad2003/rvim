use std::sync::{Arc, Mutex, MutexGuard};

use crossterm::event::{self, Event, KeyCode};

use crate::{
    STORE,
    buffer::Buffer,
    exit::graceful_exit,
    mode::{Mode, change_mode},
    window::Window,
};

pub fn handle_keys(
    window: &Arc<Mutex<Window>>,
    mode: &Arc<Mutex<Mode>>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Event::Key(key_event) = event::read()? {
        let mut mode_guard = mode.lock().unwrap();
        match &mut *mode_guard {
            Mode::Normal => {
                return handle_normal(key_event, window, &mut mode_guard);
            }
            Mode::Insert => {
                return handle_insert(key_event, window, &mut mode_guard);
            }
            Mode::Command => {
                return handle_command(key_event, window, &mut mode_guard);
            }
        }
    }
    Ok(false)
}

fn handle_command(
    key_event: event::KeyEvent,
    window: &Arc<Mutex<Window>>,
    mode: &mut MutexGuard<'_, Mode>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut redraw = true;
    match key_event.code {
        KeyCode::Esc => change_mode(mode, Mode::Normal),
        KeyCode::Char(c) => {
            let mut store = STORE.write().unwrap();
            store.command.push(c);
        }
        KeyCode::Enter => {
            let mut store = STORE.write().unwrap();
            let command = store.command.clone();
            store.command.clear();
            let command = command.trim();
            match command {
                "q" => graceful_exit(0)?,
                "w" => {
                    let buffer = window.lock().unwrap().buffer.clone();
                    let mut buffer_guard = buffer.lock().unwrap();
                    match buffer_guard.fule_path.clone() {
                        Some(_) => {
                            buffer_guard.save()?;
                        }
                        None => {
                            eprintln!("No file path set for saving.");
                        }
                    }
                }
                _ if command.starts_with("w ") => {
                    let file_path = command[2..].trim();
                    let buffer = window.lock().unwrap().buffer.clone();
                    buffer.lock().unwrap().set_file_path(file_path);
                    buffer.lock().unwrap().save()?;
                }
                _ if command.starts_with("e ") => {
                    let file_path = command[2..].trim();
                    let buffer = Buffer::load_from_file(file_path)?;
                    window
                        .lock()
                        .unwrap()
                        .set_buffer(Arc::new(Mutex::new(buffer)));
                }
                "e" => {}
                _ => {
                    println!("Unknown command: {command}");
                }
            }
            change_mode(mode, Mode::Normal);
        }
        KeyCode::Backspace => {
            let mut store = STORE.write().unwrap();
            if !store.command.is_empty() {
                store.command.pop();
            } else {
                change_mode(mode, Mode::Normal);
            }
        }

        _ => {
            redraw = false;
        }
    }
    Ok(redraw)
}
fn handle_normal(
    key_event: event::KeyEvent,
    window: &Arc<Mutex<Window>>,
    mode: &mut MutexGuard<'_, Mode>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut redraw = true;
    match key_event.code {
        KeyCode::Char('c') if key_event.modifiers == event::KeyModifiers::CONTROL => {
            graceful_exit(0)?;
        }
        KeyCode::Char('i') => {
            change_mode(mode, Mode::Insert);
        }
        KeyCode::Char(':') => {
            change_mode(mode, Mode::Command);
        }
        KeyCode::Up => window.lock().unwrap().cursor_up(),
        KeyCode::Down => window.lock().unwrap().cursor_down(),
        KeyCode::Left => window.lock().unwrap().cursor_left(),
        KeyCode::Right => window.lock().unwrap().cursor_right(),
        _ => {
            redraw = false;
        }
    }
    Ok(redraw)
}

fn handle_insert(
    key_event: event::KeyEvent,
    window: &Arc<Mutex<Window>>,
    mode: &mut MutexGuard<'_, Mode>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut redraw = true;
    match key_event.code {
        KeyCode::Char('c') if key_event.modifiers == event::KeyModifiers::CONTROL => {
            graceful_exit(0)?;
        }
        KeyCode::Char(c) => {
            window.lock().unwrap().type_char(c);
        }
        KeyCode::Enter => {
            window.lock().unwrap().type_char('\n');
        }
        KeyCode::Backspace => {
            window.lock().unwrap().backspace();
        }
        KeyCode::Up => window.lock().unwrap().cursor_up(),
        KeyCode::Down => window.lock().unwrap().cursor_down(),
        KeyCode::Left => window.lock().unwrap().cursor_left(),
        KeyCode::Right => window.lock().unwrap().cursor_right(),
        KeyCode::Esc => change_mode(mode, Mode::Normal),
        _ => {
            redraw = false;
        }
    }
    Ok(redraw)
}
