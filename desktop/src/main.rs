mod keybind;
mod shell;
fn main() {
    let normalized_key_test1 = keybind::normalize("Ctrl+Alt+T");

    println!("Normalized key: {}", normalized_key_test1);
}