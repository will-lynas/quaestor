use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use dotenv::from_path;
use std::env;
use std::path::Path;
use sqlx::sqlite::SqlitePool;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Add a transaction. Usage: /add <description> [amount]")]
    Add(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<Command>()
            .endpoint(answer),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    pool: SqlitePool,
) -> ResponseResult<()> {
    match cmd {
        Command::Add(args) => add_command(bot, msg, args, pool).await?,
    }
    Ok(())
}

async fn add_command(bot: Bot, msg: Message, args: String, pool: SqlitePool) -> ResponseResult<()> {
    let (description, amount) = parse_add_args(&args);

    if description.is_empty() {
        bot.send_message(msg.chat.id, "❌ Description cannot be empty. Usage: /add <description> [amount]").await?;
        return Ok(());
    }

    let chat_id = msg.chat.id.0;
    let user = msg.from.unwrap();
    let user_id = user.id.0 as i64;

    match sqlx::query!(
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
    .await
    {
        Ok(_) => {
            let response = format!(
                "✅ Recorded transaction of {:.2} with description '{}' from user {}",
                amount,
                description,
                user.first_name
            );
            bot.send_message(msg.chat.id, response).await?;
        },
        Err(e) => {
            log::error!("Database error: {}", e);
            bot.send_message(msg.chat.id, "❌ Failed to record the transaction. Please try again later.").await?;
        },
    }

    Ok(())
}

fn parse_add_args(args: &str) -> (String, f64) {
    let default_amount = 420.69;
    let parts: Vec<&str> = args.rsplitn(2, ' ').collect();

    match parts.len() {
        0 => (String::new(), default_amount),
        1 => (parts[0].trim().to_string(), default_amount),
        _ => {
            let amount_str = parts[0].trim();
            let description = parts[1].trim();

            let amount = amount_str.parse::<f64>().unwrap_or_else(|_| {
                log::warn!("Invalid amount provided: {}. Using default.", amount_str);
                default_amount
            });

            (description.to_string(), amount)
        }
    }
}
