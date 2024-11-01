use std::fs::File;
use std::io::{Write, BufWriter}; 

// Function to create a  CSV file
fn create_csv() -> std::io::Result<()> {
    let mut file = File::create("sample.csv")?;
    writeln!(file, "Name,Age,City")?;
    writeln!(file, "Alice,30,New York")?;
    writeln!(file, "Bob,25,Los Angeles")?;
    writeln!(file, "Charlie,35,Chicago")?;
    Ok(())
}

// Function to create a  JSON file
fn create_json() -> std::io::Result<()> {
    let mut file = File::create("sample.json")?;
    writeln!(file, "{{\"name\": \"Alice\", \"age\": 30, \"city\": \"New York\"}}")?;
    Ok(())
}

// Main function 
fn main() -> std::io::Result<()> {
    // Function calls
    create_csv()?;
    create_json()?;
    println!("Sample files created successfully.");
    Ok(())
}