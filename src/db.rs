use sqlx::SqlitePool;

pub struct Transaction {
    pub user_id: i64,
    pub description: Option<String>,
    pub amount: Option<f64>,
}

pub struct DB<'a> {
    pool: &'a SqlitePool,
}

impl<'a> DB<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_transactions(&self, chat_id: i64) -> Vec<Transaction> {
        sqlx::query_as!(
            Transaction,
            r#"
        SELECT userID as "user_id!", description, amount
        FROM transactions
        WHERE chatID = ?
        "#,
            chat_id
        )
        .fetch_all(self.pool)
        .await
        .unwrap()
    }

    pub async fn reset_chat(&self, chat_id: i64) {
        sqlx::query!(
            r#"
        DELETE FROM transactions
        WHERE chatID = ?
        "#,
            chat_id
        )
        .execute(self.pool)
        .await
        .unwrap();
    }
}
