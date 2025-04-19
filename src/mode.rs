use std::sync::MutexGuard;

#[derive(Debug)]
pub enum Mode {
    Insert,
    Normal,
    Command,
}

pub fn change_mode(mode: &mut MutexGuard<'_, Mode>, new_mode: Mode) {
    **mode = new_mode;
}
