init:
    cargo install sqlx-cli

deploy:
    just init
    just db-migrate

db-migrate:
    sqlx database create
    sqlx migrate run --source migrations
