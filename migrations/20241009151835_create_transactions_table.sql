CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chatID INTEGER,
    userID INTEGER,
    description TEXT,
    amount REAL  
);
