use sobriquet::error::{exit_code_from_error, format_error};
use std::process::ExitCode;

fn main() -> ExitCode {
    match sobriquet::run() {
        Ok(code) => code,
        Err(e) => {
            // Use the improved error formatting with the trait
            eprintln!("{}", format_error(&e));
            exit_code_from_error(&e)
        }
    }
}
