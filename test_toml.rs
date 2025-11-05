use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct TestAffordance {
    name: String,
    connects_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TestPlace {
    name: String,
    affordances: Vec<TestAffordance>,
}

#[derive(Debug, Deserialize)]
struct TestBreadboard {
    name: String,
    created: String,
    places: Vec<TestPlace>,
}

fn main() {
    let content = fs::read_to_string("90s-personal-website.toml").unwrap();
    let parsed: Result<TestBreadboard, toml::de::Error> = toml::from_str(&content);
    
    match parsed {
        Ok(board) => {
            println!("Successfully parsed: {:#?}", board);
        }
        Err(e) => {
            println!("Error parsing TOML: {}", e);
        }
    }
}
