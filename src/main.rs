use std::{
    env,
    path::Path,
};

use dotenv::from_path;
use dptree::case;
use sqlx::sqlite::SqlitePool;
use teloxide::{
    dispatching::{
        dialogue,
        dialogue::InMemStorage,
        UpdateFilterExt,
        UpdateHandler,
    },
    prelude::*,
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// The following commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text
    Help,
    /// Add a transaction to the ledger
    Add,
    /// Display the ledger
    Display,
    /// Reset the ledger
    Reset,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveAmount,
    ReceiveTitle {
        amount: f64,
    },
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

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![pool, InMemStorage::<State>::new()])
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = teloxide::filter_command::<Command, _>().branch(
        case![State::Start]
            .branch(case![Command::Help].endpoint(help))
            .branch(case![Command::Display].endpoint(display))
            .branch(case![Command::Reset].endpoint(reset))
            .branch(case![Command::Add].endpoint(start_add_dialogue)),
    );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveAmount].endpoint(receive_amount))
        .branch(case![State::ReceiveTitle { amount }].endpoint(receive_title));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await
        .unwrap();

    Ok(())
}

async fn display(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    let chat_id = msg.chat.id.0;

    let transactions = sqlx::query!(
        r#"
        SELECT userID as "user_id!", description as "description!", amount as "amount!"
        FROM transactions
        WHERE chatID = ?
        "#,
        chat_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    if transactions.is_empty() {
        bot.send_message(msg.chat.id, "No transactions found")
            .await
            .unwrap();
    } else {
        let mut lines = Vec::new();

        for tx in transactions {
            let line = format!("User {}: {} - {}", tx.user_id, tx.description, tx.amount);
            lines.push(line);
        }

        let response = lines.join("\n");

        bot.send_message(msg.chat.id, response).await.unwrap();
    }

    Ok(())
}

async fn reset(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    let chat_id = msg.chat.id.0;

    sqlx::query!(
        r#"
        DELETE FROM transactions
        WHERE chatID = ?
        "#,
        chat_id
    )
    .execute(&pool)
    .await
    .unwrap();

    bot.send_message(msg.chat.id, "All transactions have been reset")
        .await
        .unwrap();

    Ok(())
}

async fn start_add_dialogue(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Enter amount:").await?;
    dialogue.update(State::ReceiveAmount).await.unwrap();
    Ok(())
}

async fn receive_amount(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<f64>()) {
        Some(Ok(amount)) => {
            bot.send_message(msg.chat.id, "Enter title:").await?;
            dialogue
                .update(State::ReceiveTitle { amount })
                .await
                .unwrap();
        }
        _ => {
            bot.send_message(msg.chat.id, "Send me a number.").await?;
        }
    }

    Ok(())
}

async fn receive_title(
    bot: Bot,
    dialogue: MyDialogue,
    amount: f64,
    msg: Message,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.clone().text() {
        Some(title) => {
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
                title,
                amount
            )
            .execute(&pool)
            .await
            .unwrap();

            bot.send_message(
                msg.chat.id,
                format!(
                    "Recorded transaction of {} amount with description '{}' from user {}",
                    amount, title, user.first_name
                ),
            )
            .await
            .unwrap();

            dialogue.exit().await.unwrap();
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}
