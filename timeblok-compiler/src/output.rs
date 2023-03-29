use std::fmt::Display;

pub fn warn<T: Display>(msg: T) {
    eprintln!("[Warning] {}", msg);
}
