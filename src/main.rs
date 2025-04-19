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
use settings::Store;
use std::{
    cmp::{max, min},
    io::{Stdout, stdout},
    sync::{Arc, Condvar, Mutex, RwLock},
};
use window::Window;

pub static STORE: RwLock<Store> = RwLock::new(Store {
    command: String::new(),
});

fn draw(
    content: &[String],
    topline: usize,
    end: usize,
    leftcol: usize,
    right_visible: usize,
    filler: usize,
    mode: &Arc<Mutex<Mode>>,
) {
    for (i, line) in content[topline..end].iter().enumerate() {
        let line_length = line.len();
        let line_end = min(line_length, right_visible);
        let line = &line[leftcol..line_end];
        print!("{:>3} {}", topline + i + 1, line);
        // if filler != 0 || topline + i + 1 != end {
        print!("\r\n");
        // }
    }
    for i in 0..filler {
        print!("~");
        // if i != filler - 1 {
        print!("\r\n");
        // }
    }
    let command_text = STORE.read().unwrap().command.clone();
    let message = match *mode.lock().unwrap() {
        Mode::Normal => String::new(),
        Mode::Insert => String::from("-- INSERT --"),
        Mode::Command => format!(":{}", command_text),
    };
    print!("{}", message);
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
    mode: Arc<Mutex<Mode>>,
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
                &mode,
            );
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
    let buffer = Arc::clone(buffers.first().unwrap());
    let (width, height) = crossterm::terminal::size().unwrap();
    let window = Arc::new(Mutex::new(Window::new(
        Arc::clone(&buffer),
        max(width - 4, 0) as usize,
        max(height - 1, 0) as usize,
    )));

    let mode = Arc::new(Mutex::new(mode::Mode::Normal));

    terminal::enable_raw_mode()?;

    let redraw_clone = Arc::clone(&redraw_needed);
    let window_clone = Arc::clone(&window);
    let mode_clone = Arc::clone(&mode);
    let key_handler = std::thread::spawn(move || {
        loop {
            let res = handle_keys(&window_clone, &mode_clone);
            match res {
                Err(e) => {
                    println!("Error handling keys: {}", e);
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

    let redraw_clone = Arc::clone(&redraw_needed);
    redraw(redraw_clone, window, stdout, mode)?;

    key_handler
        .join()
        .expect("I don't know what happened, very very bad.");
    Ok(())
}
