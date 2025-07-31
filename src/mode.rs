use crate::STORE;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
    Command,
}

pub fn change_mode(new_mode: Mode) {
    let mut store = STORE.write().unwrap();
    store.mode = new_mode;
}
