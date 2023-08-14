use anyhow::Result as AnyResult;
use jwalk::WalkDir;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::{env::args, fs::File, io::Write, path::Path};

const EXTENSIONS: [&str; 2] = ["md", "markdown"];
const IGNORED_DIRECTORIES: [&str; 6] = [
    "archive",
    "embedded",
    "embedded-hal",
    "atmel",
    "node_modules",
    "STM32",
];

fn ends_with_extension(path: &str) -> bool {
    EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LinkTag {
    url: String,
    title: String,
    path: String,
}

impl LinkTag {
    fn new(url: String, title: String, path: String) -> Self {
        Self { url, title, path }
    }
}

fn get_document_link(document: &str, path: String) -> Vec<LinkTag> {
    let mut links: Vec<LinkTag> = Vec::new();
    let parser = Parser::new_ext(document, Options::empty()).map(|event| match event {
        Event::Start(Tag::Link(a, url, title)) => {
            links.push(LinkTag::new(
                url.to_string(),
                title.to_string(),
                path.to_owned(),
            ));
            Event::Start(Tag::Link(a, url, title))
        }
        _ => event,
    });
    let mut res = String::new();
    html::push_html(&mut res, parser);
    links
}

enum LinkCheckError {
    CannotGetLink,
}

async fn ping_external_link(url: &str) -> std::result::Result<u32, LinkCheckError> {
    let resp = reqwest::get(url).await;
    if let Some(status_code) = resp.as_ref().ok().map(|r| r.status().as_u16()) {
        return match status_code {
            200..=399 => Ok(status_code as u32),
            _ => Err(LinkCheckError::CannotGetLink),
        };
    }
    return Err(LinkCheckError::CannotGetLink);
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let mut dead_external_links: Vec<LinkTag> = Vec::new();
    let mut dead_internal_links: Vec<LinkTag> = Vec::new();
    let path = args()
        .skip(1)
        .take(1)
        .next()
        .unwrap_or_else(|| "./tests".to_string());

    let forbidden_link_prefix = std::env::var("FORBIDDEN_LINK_PREFIX").unwrap_or_default();

    println!("FORBIDDEN_LINK_PREFIX: {:?}", forbidden_link_prefix);
    for entry in WalkDir::new(path).sort(true) {
        if let Ok(file_like) = entry {
            if ends_with_extension(file_like.path().to_str().unwrap()) {
                let path_of_file = file_like.path().display().to_string();
                if IGNORED_DIRECTORIES
                    .iter()
                    .any(|dir| path_of_file.contains(dir) || path_of_file.starts_with(dir))
                {
                    continue;
                }
                println!("Checking: {:?}", file_like.path());
                let file_content = std::fs::read_to_string(file_like.path())?;
                for link in get_document_link(
                    &file_content,
                    file_like.path().to_owned().display().to_string(),
                ) {
                    if link.url.starts_with("..") || link.url.starts_with("/") {
                        match Path::new(&link.url).canonicalize() {
                            Ok(path) => {
                                if !path.exists() {
                                    dead_internal_links.push(link);
                                }
                            }
                            Err(_) => {
                                dead_internal_links.push(link);
                            }
                        }
                    } else if (link.url.starts_with("http://") || link.url.starts_with("https://"))
                        && !link.url.contains("localhost")
                    {
                        match ping_external_link(&link.url).await {
                            Err(_) => {
                                dead_external_links.push(link);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    let external: serde_json::Value = serde_json::json!({
        "dead_external_links": dead_external_links,
        "dead_internal_links": dead_internal_links,
    });
    let mut file = File::create("dead_links.json")?;
    file.write_all(external.to_string().as_bytes())?;
    Ok(())
}
