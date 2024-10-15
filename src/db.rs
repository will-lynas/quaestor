use sqlx::SqlitePool;

pub struct Transaction {
    pub user_id: i64,
    pub title: String,
    pub amount: f64,
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
        SELECT userID as "user_id!", title, amount
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

    pub async fn add_transaction(&self, transaction: Transaction) {
        sqlx::query!(
            r#"
                INSERT INTO transactions (chatID, userID, title, amount)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            self.chat_id,
            transaction.user_id,
            transaction.title,
            transaction.amount
        )
        .execute(self.pool)
        .await
        .unwrap();
    }

    pub async fn update_user(&self, user_id: i64, username: &str) {
        sqlx::query!(
            r#"
            INSERT INTO users (user_id, username)
            VALUES (?1, ?2)
            ON CONFLICT(user_id) DO UPDATE SET username = ?2
            "#,
            user_id,
            username
        )
        .execute(self.pool)
        .await
        .unwrap();
    }
}
