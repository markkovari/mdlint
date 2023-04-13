use jwalk::WalkDir;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::{env::args, io::Result};

const EXTENSIONS: [&str; 2] = ["md", "markdown"];

fn ends_with_extension(path: &str) -> bool {
    EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

fn get_document_link(document: &str) -> Vec<(String, String)> {
    let mut links: Vec<(String, String)> = Vec::new();
    let parser = Parser::new_ext(document, Options::empty()).map(|event| match event {
        Event::Start(Tag::Link(a, url, title)) => {
            links.push((url.to_string(), title.to_string()));
            Event::Start(Tag::Link(a, url, title))
        }
        _ => event,
    });
    let mut res = String::new();
    html::push_html(&mut res, parser);
    links
}

fn main() -> Result<()> {
    let path = args()
        .skip(1)
        .take(1)
        .next()
        .unwrap_or_else(|| "./tests".to_string());

    for entry in WalkDir::new(path).sort(true) {
        if let Ok(entry) = entry {
            if ends_with_extension(entry.path().to_str().unwrap()) {
                println!("{}", entry.path().display());
                let file_content = std::fs::read_to_string(entry.path())?;
                for link in get_document_link(&file_content) {
                    println!("Link: {:?}", link);
                }
            }
        }
    }
    Ok(())
}
