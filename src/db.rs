use sqlx::SqlitePool;

pub struct Transaction {
    pub user_id: i64,
    pub description: Option<String>,
    pub amount: Option<f64>,
}

pub struct DB<'a> {
    pool: &'a SqlitePool,
    chat_id: i64,
}

impl<'a> DB<'a> {
    pub fn new(pool: &'a SqlitePool, chat_id: i64) -> Self {
        Self { pool, chat_id }
    }

    pub async fn get_transactions(&self) -> Vec<Transaction> {
        sqlx::query_as!(
            Transaction,
            r#"
        SELECT userID as "user_id!", description, amount
        FROM transactions
        WHERE chatID = ?
        "#,
            self.chat_id
        )
        .fetch_all(self.pool)
        .await
        .unwrap()
    }

    pub async fn reset_chat(&self) {
        sqlx::query!(
            r#"
        DELETE FROM transactions
        WHERE chatID = ?
        "#,
            self.chat_id
        )
        .execute(self.pool)
        .await
        .unwrap();
    }
}
