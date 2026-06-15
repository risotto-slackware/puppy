mod puppy;

fn main() {
    if let Err(error) = puppy::run() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
