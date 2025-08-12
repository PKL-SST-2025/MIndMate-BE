CREATE TABLE moods (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    mood VARCHAR(50) NOT NULL CHECK (mood IN ('very sad', 'sad', 'neutral', 'happy', 'very happy')),
    emoji VARCHAR(10) NOT NULL,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);