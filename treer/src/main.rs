fn main() {
    println!("Hello, world!");

    if let Err(e) = treer::get_args().and_then(treer::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
