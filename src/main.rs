use teloxide::prelude::*;
use dotenv::from_path;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() {
    let dotenv_path = Path::new(".env");
    from_path(dotenv_path).expect("Failed to read .env file");

    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot_token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found in .env file");
    let bot = Bot::new(bot_token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}

