CREATE TABLE factoids (
    id TEXT PRIMARY KEY NOT NULL,
    description TEXT NOT NULL,
    locked BOOLEAN NOT NULL DEFAULT 'f'
)