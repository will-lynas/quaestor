export DATABASE_URL := "sqlite://database.db"

install:
    cargo install sqlx-cli

init:
    just install
    just db-migrate

deploy:
    just init
    cargo build --release

db-migrate:
    sqlx database create
    sqlx migrate run --source migrations

db-reset:
    sqlx database drop
    just db-migrate
