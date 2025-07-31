use std::fs::{self, File};
use std::io::BufWriter;
use std::sync::atomic::{AtomicUsize, Ordering};

use ropey::Rope;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineTermination {
    CRLF,
    LF,
    CR,
}

pub fn get_line_seperator(terminator: &LineTermination) -> &str {
    match terminator {
        LineTermination::CRLF => "\r\n",
        LineTermination::LF => "\n",
        LineTermination::CR => "\r",
    }
}

fn line_termination_from_str(s: &str) -> LineTermination {
    match s {
        "\r\n" => LineTermination::CRLF,
        "\n" => LineTermination::LF,
        "\r" => LineTermination::CR,
        _ => panic!("Invalid line termination"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub id: usize,
    pub name: String,
    pub content: Rope,
    pub edited: bool,
    pub lines: usize,
    pub line_termination: LineTermination,
    pub fule_path: Option<String>,
}

impl Buffer {
    fn new(name: &str, content: &str) -> Self {
        let lines = content.len();
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            name: name.to_string(),
            content: Rope::from_str(content),
            edited: false,
            lines,
            line_termination: LineTermination::LF,
            fule_path: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            name: "[No Name]".to_string(),
            content: Rope::new(),
            edited: false,
            lines: 1,
            line_termination: LineTermination::LF,
            fule_path: None,
        }
    }

    pub fn load_from_file(file_path: &str) -> std::io::Result<Self> {
        let content = fs::read_to_string(file_path)?;
        let name = Buffer::get_name_from_file_path(file_path);
        let mut buf = Self::new(name, &content);
        buf.set_file_path(file_path);
        Ok(buf)
    }

    pub fn save_to_file(&self, file_path: &str) -> std::io::Result<()> {
        let writer = BufWriter::new(File::create_new(file_path).unwrap());
        self.content.write_to(writer)
    }

    fn get_name_from_file_path(file_path: &str) -> &str {
        file_path.split('/').next_back().unwrap_or(file_path)
    }

    pub fn set_file_path(&mut self, file_path: &str) {
        self.fule_path = Some(file_path.to_string());
        let name = Buffer::get_name_from_file_path(file_path);
        self.name = name.to_string();
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file_path) = &self.fule_path {
            return self.save_to_file(file_path);
        }
        Ok(())
    }
}
