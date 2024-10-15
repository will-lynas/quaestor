use teloxide::utils::markdown;

use crate::db::DB;

fn format_pounds(value: f64) -> String {
    format!("£{:.2}", value)
}

pub async fn format_transaction(
    db: &DB<'_>,
    title: &str,
    amount: f64,
    user_id: i64,
    description: &str,
) -> String {
    let username = db
        .get_username(user_id)
        .await
        .unwrap_or_else(|| user_id.to_string());
    let mut formatted = format!(
        "🏷️ {}\n💰 {}\n🥷 [{}](tg://user?id={})",
        markdown::escape(title),
        markdown::escape(&format_pounds(amount)),
        markdown::escape(&username),
        user_id
    );
    if !description.is_empty() {
        formatted.push_str(&format!("\n📝 {}", markdown::escape(description)));
    }
    formatted
}
