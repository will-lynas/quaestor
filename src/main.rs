use dotenv::from_path;
use sqlx::sqlite::SqlitePool;
use std::env;
use std::path::Path;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

/// The following commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text
    Help,
    /// Add a transaction to the ledger
    #[command(parse_with = "split")]
    Add { amount: f64, desc: String },
}

#[tokio::main]
async fn main() {
    let dotenv_path = Path::new(".env");
    from_path(dotenv_path).expect("Failed to read .env file");

    pretty_env_logger::init();
    log::info!("Starting the bot...");

    let bot_token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found in .env file");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in .env file");

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let bot = Bot::new(bot_token);

    Command::repl(bot, move |bot: Bot, msg: Message, cmd: Command| {
        let pool = pool.clone();
        async move {
            handle_cmd(bot, msg, cmd, pool).await;
            Ok(())
        }
    })
    .await;
}

async fn handle_cmd(bot: Bot, msg: Message, cmd: Command, pool: SqlitePool) {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await
                .unwrap();
        }
        Command::Add { amount, desc } => {
            let chat_id = msg.chat.id.0;
            let user = msg.from.unwrap();
            let user_id = user.id.0 as i64;

            sqlx::query!(
                r#"
                INSERT INTO transactions (chatID, userID, description, amount)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                chat_id,
                user_id,
                desc,
                amount
            )
            .execute(&pool)
            .await
            .unwrap();

            bot.send_message(
                msg.chat.id,
                format!(
                    "Recorded transaction of {} amount with description '{}' from user {}",
                    amount, desc, user.first_name
                ),
            )
            .await
            .unwrap();
        }
    }
}
