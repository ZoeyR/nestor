CREATE TABLE factoids (
    id INTEGER PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    intent TEXT NOT NULL,
    description TEXT NOT NULL,
    nickname TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    locked BOOLEAN NOT NULL DEFAULT 'f'
);