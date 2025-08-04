CREATE TABLE moods (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    date DATE NOT NULL DEFAULT (DATE('now')),
    mood TEXT NOT NULL CHECK (mood IN ('very sad', 'sad', 'neutral', 'happy', 'very happy')),
    emoji TEXT NOT NULL,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);