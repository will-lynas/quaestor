ALTER TABLE transactions RENAME TO transactions_old;

CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chatID INTEGER NOT NULL,
    userID INTEGER NOT NULL,
    title TEXT NOT NULL,  -- Renamed column
    amount REAL NOT NULL
);

INSERT INTO transactions (id, chatID, userID, title, amount)
SELECT id, chatID, userID, description, amount
FROM transactions_old;

DROP TABLE transactions_old;
