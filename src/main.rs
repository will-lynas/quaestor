use std::{
    env,
    path::Path,
};

use db::{
    Transaction,
    DB,
};
use dotenv::from_path;
use dptree::case;
use sqlx::sqlite::SqlitePool;
use teloxide::{
    dispatching::{
        dialogue::{
            self,
            InMemStorage,
        },
        UpdateFilterExt,
        UpdateHandler,
    },
    prelude::*,
    types::ParseMode::MarkdownV2,
    utils::{
        command::BotCommands,
        markdown,
    },
};
use utils::format_pounds;

mod db;
mod utils;

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
    AddStart,
    AddReceiveTitle,
    AddReceiveAmount {
        title: String,
    },
    AddReceiveDescription {
        title: String,
        amount: f64,
    },
}

// Clippy needless_return is bugged with tokio on nightly
// See https://github.com/rust-lang/rust-clippy/issues/13458
#[allow(clippy::needless_return)]
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
        case![State::AddStart]
            .branch(case![Command::Help].endpoint(help))
            .branch(case![Command::Display].endpoint(display))
            .branch(case![Command::Reset].endpoint(reset))
            .branch(case![Command::Add].endpoint(start_add_dialogue)),
    );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::AddReceiveTitle].endpoint(receive_title))
        .branch(case![State::AddReceiveAmount { title }].endpoint(receive_amount))
        .branch(
            case![State::AddReceiveDescription { title, amount }].endpoint(receive_description),
        );

    let update_user_handler =
        Update::filter_message().map_async(|msg: Message, pool: SqlitePool| async move {
            let user = msg.from.unwrap();
            let db = DB::new(&pool);
            db.update_user(user.id.0 as i64, user.username.as_deref())
                .await;
        });

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(update_user_handler)
        .branch(message_handler)
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await
        .unwrap();

    Ok(())
}

async fn display(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    let db = DB::new(&pool);

    let transactions = db.get_transactions(chat_id).await;

    if transactions.is_empty() {
        bot.send_message(msg.chat.id, "No transactions found")
            .await
            .unwrap();
    } else {
        let mut lines = Vec::new();

        for tx in transactions {
            let username = db
                .get_username(tx.user_id)
                .await
                .unwrap_or_else(|| tx.user_id.to_string());
            let mut line = format!(
                "🏷️ {}\n💰 {}\n🥷 [{}](tg://user?id={})",
                markdown::escape(&tx.title),
                markdown::escape(&format_pounds(tx.amount)),
                markdown::escape(&username),
                tx.user_id
            );
            if !tx.description.is_empty() {
                line.push_str(&format!("\n📝 {}", markdown::escape(&tx.description)));
            }
            lines.push(line);
        }

        let response = lines.join("\n\n");

        bot.send_message(msg.chat.id, response)
            .parse_mode(MarkdownV2)
            .await
            .unwrap();
    }

    Ok(())
}

async fn reset(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    DB::new(&pool).reset_chat(chat_id).await;

    bot.send_message(msg.chat.id, "All transactions have been reset")
        .await
        .unwrap();

    Ok(())
}

async fn start_add_dialogue(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Enter title:").await?;
    dialogue.update(State::AddReceiveTitle).await.unwrap();
    Ok(())
}

async fn receive_title(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(title) => {
            bot.send_message(msg.chat.id, "Enter amount:").await?;
            dialogue
                .update(State::AddReceiveAmount {
                    title: title.to_string(),
                })
                .await
                .unwrap();
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text").await?;
        }
    }

    Ok(())
}

async fn receive_amount(
    bot: Bot,
    dialogue: MyDialogue,
    title: String,
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<f64>()) {
        Some(Ok(amount)) => {
            bot.send_message(msg.chat.id, "Enter description:").await?;
            dialogue
                .update(State::AddReceiveDescription { title, amount })
                .await
                .unwrap();
        }
        _ => {
            bot.send_message(msg.chat.id, "Send me a number").await?;
        }
    }

    Ok(())
}

async fn receive_description(
    bot: Bot,
    dialogue: MyDialogue,
    (title, amount): (String, f64),
    msg: Message,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text() {
        Some(description) => {
            let chat_id = msg.chat.id.0;
            let user = msg.clone().from.unwrap();
            let user_id = user.id.0 as i64;
            let name = user.username.unwrap_or(user_id.to_string());

            let description = if description == "-" {
                String::new()
            } else {
                description.to_string()
            };

            let transaction = Transaction {
                user_id,
                title: title.clone(),
                amount,
                description: description.clone(),
            };

            DB::new(&pool).add_transaction(chat_id, transaction).await;

            let mut response = format!(
                "*Added transaction*\n\n 🏷️ {}\n 💰 {}\n 🥷 [{}](tg://user?id={})",
                markdown::escape(&title),
                markdown::escape(&format_pounds(amount)),
                markdown::escape(&name),
                user_id
            );

            if !description.is_empty() {
                response.push_str(&format!("\n📝 {}", markdown::escape(&description)));
            }

            bot.send_message(msg.chat.id, response)
                .parse_mode(MarkdownV2)
                .await
                .unwrap();

            dialogue.exit().await.unwrap();

            Ok(())
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text").await?;
            Ok(())
        }
    }
}
