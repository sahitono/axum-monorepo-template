set shell := ["sh", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set dotenv-load

# build for relase
release:
    cargo build --bin {{project_name}} --release

# run in dev mode pass config file location
run_dev *ARGS:
    cargo run --package {{project_name}} --bin {{project_name}} -- --config {{ ARGS }}
