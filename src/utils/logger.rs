use std::fmt::Display;

/* Imports */
/* Constants */
const TITLE_WIDTH: usize = 14;
lazy_static::lazy_static! {
    static ref DEBUG: bool = env!("DEBUG").parse::<bool>().unwrap();
}

/// Logs if in debug mode
pub fn log(title: &str, text: impl Display) -> () {
    let padding = (TITLE_WIDTH - 2).checked_sub(title.len()).unwrap_or(0);

    if *DEBUG {
        println!("[{title}]{} {}", " ".repeat(padding), text)
    }
}
