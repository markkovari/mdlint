use comrak::{
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use jwalk::WalkDir;
use std::{env::args, io::Result};

const EXTENSIONS: [&str; 2] = ["md", "markdown"];

fn ends_with_extension(path: &str) -> bool {
    EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

fn get_document_title(document: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, document, &ComrakOptions::default());

    for node in root.children() {
        let header = match node.data.clone().into_inner().value {
            NodeValue::Heading(c) => c,
            _ => continue,
        };
        if header.level != 1 {
            continue;
        }
        let mut text = String::new();
        collect_text(node, &mut text);
        return text;
    }

    "Untitled Document".to_string()
}

fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut String) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.push_str(literal)
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
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
                let title = get_document_title(&file_content);
                println!("title: {}", title);
            }
        }
    }
    Ok(())
}
