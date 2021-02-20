use std::error::Error;

pub fn log_if_error<T, E: Error>(msg: &str, result: Result<T, E>) -> Option<T> {
    match result {
        Ok(o) => Some(o),
        Err(e) => {
            eprintln!("{}: {}", msg, e);
            None
        }
    }
}
