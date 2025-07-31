mod buffer;
mod cursor;
mod exit;
mod key_handling;
mod mode;
mod settings;
mod window;

use buffer::Buffer;
use crossterm::{execute, terminal};
use exit::graceful_exit;
use key_handling::handle_keys;
use mode::Mode;
use ropey::Rope;
use settings::Store;
use std::{
    cmp::min,
    io::{Stdout, Write, stdout},
    sync::{Arc, Condvar, Mutex, RwLock},
};
use window::Window;

use crate::buffer::{
    LineTermination::{CRLF, LF},
    get_line_seperator,
};

pub static STORE: RwLock<Store> = RwLock::new(Store {
    command: String::new(),
    mode: Mode::Normal,
});

fn draw(
    content: &Rope,
    topline: usize,
    end: usize,
    leftcol: usize,
    right_visible: usize,
    filler: usize,
) -> Result<(), std::io::Error> {
    let nl = get_line_seperator(&CRLF);

    let lfnl = get_line_seperator(&LF);

    let mut lock = stdout().lock();
    for i in topline..end {
        let line = content.line(i);
        let line_end = min(line.len_chars(), right_visible);
        let mut line = line.slice(leftcol..line_end).to_string();
        if line.ends_with(lfnl) {
            line.pop();
        }

        let line_txt = format!("{:>3} {}{}", topline + i + 1, line, nl);
        write!(lock, "{line_txt}")?;
    }

    for _ in 0..filler {
        write!(lock, "~{nl}")?;
    }
    let store = STORE.read().unwrap();
    let command_text = &store.command;
    let message = match store.mode {
        Mode::Normal => String::new(),
        Mode::Insert => String::from("-- INSERT --"),
        Mode::Command => format!(":{command_text}"),
    };

    write!(lock, "{message}")?;
    stdout().flush()
}

fn move_cursor(cursor: cursor::Cursor) -> Result<(), std::io::Error> {
    execute!(
        stdout(),
        crossterm::cursor::MoveTo(4 + cursor.col as u16, cursor.row as u16)
    )
}

fn hide_cursor(stdout: &mut Stdout) -> Result<(), std::io::Error> {
    execute!(stdout, crossterm::cursor::Hide)
}

fn show_cursor(stdout: &mut Stdout) -> Result<(), std::io::Error> {
    execute!(stdout, crossterm::cursor::Show)
}

fn redraw(
    redraw_needed: Arc<(Mutex<bool>, Condvar)>,
    window: Arc<Mutex<Window>>,
    mut stdout: Stdout,
) -> Result<(), std::io::Error> {
    loop {
        hide_cursor(&mut stdout)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;
        let cursor: cursor::Cursor;
        {
            let window = window.lock().unwrap();
            cursor = window.cursor;
            let (topline, end, leftcol, right_visible, filler) = window.visible_range();
            let buffer = window.buffer.lock().unwrap();
            draw(
                &buffer.content,
                topline,
                end,
                leftcol,
                right_visible,
                filler,
            )?;
            move_cursor(cursor)?;
        }
        show_cursor(&mut stdout)?;
        let (lock, cvar) = &*redraw_needed;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        *started = false;
    }
}

fn main() -> Result<(), std::io::Error> {
    let redraw_needed = Arc::new((Mutex::new(false), Condvar::new()));
    let buffers: Vec<Arc<Mutex<Buffer>>> = vec![Mutex::new(Buffer::empty()).into()];
    let buffer = buffers.first().unwrap().clone();
    let (width, height) = crossterm::terminal::size().unwrap();
    let window = Arc::new(Mutex::new(Window::new(
        buffer.clone(),
        width.saturating_sub(4).into(),
        height.saturating_sub(1).into(),
    )));

    terminal::enable_raw_mode()?;

    let redraw_clone = redraw_needed.clone();
    let window_clone = window.clone();
    let key_handler = std::thread::spawn(move || {
        loop {
            let res = handle_keys(&window_clone);
            match res {
                Err(e) => {
                    println!("Error handling keys: {e}");
                    let _ = graceful_exit(-1);
                }
                Ok(redraw) => {
                    if redraw {
                        let (lock, cvar) = &*redraw_clone;
                        let mut redraw_needed = lock.lock().unwrap();
                        *redraw_needed = true;
                        cvar.notify_one();
                    }
                }
            }
        }
    });

    let stdout = stdout();

    let redraw_clone = redraw_needed.clone();
    redraw(redraw_clone, window, stdout)?;

    key_handler
        .join()
        .expect("I don't know what happened, very very bad.");
    Ok(())
}
