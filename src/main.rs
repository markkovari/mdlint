use jwalk::WalkDir;
use markdown::mdast::Node;
use std::{env::args, io::Result};

const EXTENSIONS: [&str; 2] = ["md", "markdown"];

type LinkURI = String;

fn ends_with_extension(path: &str) -> bool {
    EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

fn extract_links(content: String) -> Result<Vec<LinkURI>> {
    let mut links: Vec<LinkURI> = Vec::new();

    let parsed = markdown::to_mdast(&content, &markdown::ParseOptions::gfm());
    if parsed.is_err() {
        eprintln!("Failed to parse markdown: {}", content);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to parse markdown",
        ));
    }
    let local = parsed.unwrap();
    let children = local.children().unwrap();
    for node in children.iter() {
        match node {
            Node::Link(link) => {
                links.push(link.url.to_string());
            }
            _ => {}
        }
    }
    Ok(links)
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
                let file_content = std::fs::read_to_string(entry.path())?;
                if let Ok(links) = extract_links(file_content) {
                    for link in links {
                        println!("  {}", link);
                    }
                }
            }
        }
    }
    Ok(())
}
