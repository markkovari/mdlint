use anyhow::Result as AnyResult;
use jwalk::WalkDir;
use log::{error, info};
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::{
    env::args,
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

const EXTENSIONS: [&str; 2] = ["md", "markdown"];
const IGNORED_DIRECTORIES: [&str; 7] = [
    "archive",
    "embedded",
    "embedded-hal",
    "atmel",
    "node_modules",
    "STM32",
    "legacy",
];

fn ends_with_extension(path: &str) -> bool {
    EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        info!("Status code: {} of {}", status_code, url);
        return match status_code {
            200..=399 => Ok(status_code as u32),
            _ => Err(LinkCheckError::CannotGetLink),
        };
    }
    return Err(LinkCheckError::CannotGetLink);
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    env_logger::init();
    let mut dead_external_links: Vec<LinkTag> = Vec::new();
    let mut dead_internal_links: Vec<LinkTag> = Vec::new();
    let mut should_be_relative: Vec<LinkTag> = Vec::new();
    let path = args()
        .skip(1)
        .take(1)
        .next()
        .unwrap_or_else(|| "./tests".to_string());

    let forbidden_link_prefix = std::env::var("FORBIDDEN_LINK_PREFIX").unwrap_or_default();
    let current_repo_url = std::env::var("CURRENT_REPO_URL").unwrap_or_default();
    let requires_gh_auth = std::env::var("REQUIRES_GH_AUTH").unwrap_or_default();
    let mut already_visited = Arc::new(Mutex::new(Vec::<LinkTag>::new()));

    println!("FORBIDDEN_LINK_PREFIX: {:?}", forbidden_link_prefix);

    let (tx, mut rx) = mpsc::channel::<LinkTag>(100);
    tokio::spawn(async move {
        for entry in WalkDir::new(path).sort(true) {
            info!("Checking: {:?}", entry);
            if let Ok(file_like) = entry {
                info!("Checking file: {:?}", file_like);
                if ends_with_extension(file_like.path().to_str().unwrap()) {
                    let path_of_file = file_like.path().display().to_string();
                    if IGNORED_DIRECTORIES
                        .iter()
                        .any(|dir| path_of_file.contains(dir))
                    {
                        info!("Ignoring: {:?}", file_like.path());
                        continue;
                    }
                    info!("Checking: {:?}", file_like.path());
                    if let Ok(file_content) = std::fs::read_to_string(file_like.path()) {
                        for link in get_document_link(
                            &file_content,
                            file_like.path().to_owned().display().to_string(),
                        ) {
                            if link.url.starts_with("#") {
                                info!("Ignoring fragment: {:?}", link);
                                continue;
                            }
                            if link.url.starts_with(&current_repo_url) {
                                info!("Ignoring repo relatice link: {:?}", link);
                                should_be_relative.push(link);
                                continue;
                            }
                            if link.url.starts_with(&requires_gh_auth) {
                                info!("Ignoring, gh token is needed: {:?}", link);
                                // should_be_relative.push(link);
                                continue;
                            }
                            if link.url.starts_with("..") || link.url.starts_with("/") {
                                match Path::new(file_like.file_name())
                                    .join(&link.url)
                                    .canonicalize()
                                {
                                    Ok(path) => {
                                        if !path.exists() {
                                            error!("Cannot find internal: {:?}", link);
                                            dead_internal_links.push(link);
                                        }
                                    }
                                    Err(_) => {
                                        error!("Cannot find internal: {:?}", link);
                                        dead_internal_links.push(link);
                                    }
                                }
                            } else if (link.url.starts_with("http://")
                                || link.url.starts_with("https://"))
                                && !link.url.contains("localhost")
                            {
                                tx.send(link.clone()).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    });
    while let Some(link) = rx.recv().await {
        let mut already_visited = already_visited.lock().unwrap();
        if already_visited.iter().any(|l| l.url == link.url) {
            continue;
        }
        match ping_external_link(&link.url).await {
            Err(_) => {
                error!("Cannot find external: {:?}", link);
                dead_external_links.push(link.clone());
            }
            _ => {}
        }
        already_visited.push(link);
    }

    let already = already_visited.lock().unwrap();
    let external: serde_json::Value = serde_json::json!({
        "already": already.clone(),
    });

    let mut file = File::create("dead_links.json")?;
    file.write_all(external.to_string().as_bytes())?;
    Ok(())
}
