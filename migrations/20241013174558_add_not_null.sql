ALTER TABLE transactions RENAME TO transactions_old;

CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chatID INTEGER NOT NULL,
    userID INTEGER NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL
);

INSERT INTO transactions (id, chatID, userID, description, amount)
SELECT id, chatID, userID, description, amount
FROM transactions_old;

DROP TABLE transactions_old;
