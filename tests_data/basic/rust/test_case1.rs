/// Sample function to load a file
fn load_model() {
    println!("Magika model ⏳");
    // Simulating a delay for loading
    std::thread::sleep(std::time::Duration::from_millis(200));
    println!("Model loaded successfully!");
}

fn main() {
    // Load the model
    load_model();
}