install:
    cargo install sqlx-cli

deploy:
    just install
    just db-migrate
    cargo build --release

ci:
    just install
    just db-migrate

db-migrate:
    sqlx database create
    sqlx migrate run --source migrations

db-reset:
    sqlx database drop
    just db-migrate
