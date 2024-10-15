use teloxide::utils::markdown;

fn format_pounds(value: f64) -> String {
    format!("£{:.2}", value)
}

pub fn format_transaction(
    title: &str,
    amount: f64,
    username: &str,
    user_id: i64,
    description: &str,
) -> String {
    let mut formatted = format!(
        "🏷️ {}\n💰 {}\n🥷 [{}](tg://user?id={})",
        markdown::escape(title),
        markdown::escape(&format_pounds(amount)),
        markdown::escape(username),
        user_id
    );
    if !description.is_empty() {
        formatted.push_str(&format!("\n📝 {}", markdown::escape(description)));
    }
    formatted
}
