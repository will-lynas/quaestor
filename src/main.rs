use teloxide::prelude::*;
use dotenv::from_path;
use std::env;
use std::path::Path;
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() {
    let dotenv_path = Path::new(".env");
    from_path(dotenv_path).expect("Failed to read .env file");

    pretty_env_logger::init();
    log::info!("Starting the bot...");

    let bot_token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found in .env file");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in .env file");
    let bot = Bot::new(bot_token);

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    teloxide::repl(bot, move |_bot: Bot, msg: Message| {
        let pool = pool.clone();
        async move {
            let description = msg.text().unwrap_or("").to_string();

            let chat_id = msg.chat.id.0;
            let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
            let amount = 420.69_f64;

            sqlx::query!(
                r#"
                INSERT INTO transactions (chatID, userID, description, amount)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                chat_id,
                user_id,
                description,
                amount
            )
            .execute(&pool)
            .await.unwrap();

            Ok(())
        }
    })
    .await;
}

