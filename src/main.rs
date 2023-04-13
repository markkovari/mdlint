use jwalk::WalkDir;
use std::{env::args, io::Result};

const EXTENSIONS: [&str; 2] = ["md", "markdown"];

fn ends_with_extension(path: &str) -> bool {
    EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

fn main() -> Result<()> {
    let path = args()
        .skip(1)
        .take(1)
        .next()
        .unwrap_or_else(|| ".".to_string());

    println!("Path: {}", path);
    for entry in WalkDir::new(path).sort(true) {
        if let Ok(entry) = entry {
            if ends_with_extension(entry.path().to_str().unwrap()) {
                println!("{}", entry.path().display());
            }
        }
    }
    Ok(())
}
