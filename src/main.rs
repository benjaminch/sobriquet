use std::process::ExitCode;

fn main() -> ExitCode {
    match sobriquet::run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
