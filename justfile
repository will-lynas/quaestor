init:
    cargo install sqlx-cli

deploy:
    just init
    just db-migrate
    just build

build:
    cargo build --release

db-migrate:
    sqlx database create
    sqlx migrate run --source migrations
