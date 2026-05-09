fn main() {
    if let Err(err) = notes_grep::run() {
        eprintln!("ng: {err}");
        std::process::exit(err.exit_code());
    }
}
