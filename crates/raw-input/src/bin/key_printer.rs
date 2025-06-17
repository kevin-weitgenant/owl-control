use raw_input::RawInput;

fn main() {
    RawInput::initialize(|event| println!("{event:?}")).expect("Failed to initialize raw input");
}
