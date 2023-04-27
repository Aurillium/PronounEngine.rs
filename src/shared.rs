use chrono::prelude::*;

pub fn console_stamp() -> String {
    Utc::now().format("[%d/%m/%y %H:%M:%S] ").to_string()
}