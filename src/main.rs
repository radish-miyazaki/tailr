fn main() {
    if let Err(e) = tailr::get_cli().and_then(|cli| tailr::run(&cli)) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
