fn main() {
    match sysy::main() {
        Ok(()) => {}
        Err(e) => eprintln!("Error: {}", e),
    }
}
