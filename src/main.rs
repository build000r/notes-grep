fn main() {
    if let Err(err) = notes_grep::run() {
        let msg = err.to_string();
        if !msg.is_empty() {
            eprintln!("ng: {msg}");
        }
        std::process::exit(err.exit_code());
    }
}
