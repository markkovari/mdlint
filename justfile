build:
    cargo build
release:
    cargo build --release
release-wasm:
    cargo build --release --target wasm32-unknown-unknown

copy-wasm:
    cp target/wasm32-unknown-unknown/release/mdlint.wasm mdlint.wasm

run:
    cargo run

##locally
try:
    cargo run -- "/Users/markkovari/DEV/projects/teaching-materials"
try-ca:
    cargo run -- "/Users/markkovari/DEV/projects/teaching-materials/cognitive-apprenticeship"
try-dojo:
    cargo run -- "/Users/markkovari/DEV/projects/teaching-materials/dojo"