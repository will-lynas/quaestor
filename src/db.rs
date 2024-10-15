use sqlx::SqlitePool;

pub struct Transaction {
    pub user_id: i64,
    pub title: String,
    pub amount: f64,
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
        SELECT userID as "user_id!", title, amount
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

    pub async fn add_transaction(&self, chat_id: i64, transaction: Transaction) {
        sqlx::query!(
            r#"
                INSERT INTO transactions (chatID, userID, title, amount)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            chat_id,
            transaction.user_id,
            transaction.title,
            transaction.amount
        )
        .execute(self.pool)
        .await
        .unwrap();
    }

    pub async fn update_user(&self, user_id: i64, username: Option<&str>) {
        match username {
            Some(name) => {
                sqlx::query!(
                    r#"
                    INSERT INTO users (user_id, username)
                    VALUES (?1, ?2)
                    ON CONFLICT(user_id) DO UPDATE SET username = ?2
                    "#,
                    user_id,
                    name
                )
                .execute(self.pool)
                .await
                .unwrap();
            }
            None => {
                sqlx::query!(
                    r#"
                    INSERT INTO users (user_id)
                    VALUES (?1)
                    ON CONFLICT(user_id) DO NOTHING
                    "#,
                    user_id
                )
                .execute(self.pool)
                .await
                .unwrap();
            }
        }
    }

    pub async fn get_username(&self, user_id: i64) -> Option<String> {
        sqlx::query!(
            r#"
            SELECT username
            FROM users
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_one(self.pool)
        .await
        .unwrap()
        .username
    }
}
