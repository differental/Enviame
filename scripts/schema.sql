-- Usage
-- 1. createdb your_database_name
-- 2. psql -U your_username -d your_database_name -f schema.sql
-- 3. Change in .env, DATABASE_URL=postgresql://your_username:your_password@localhost:5432/your_database_name

-- users table
CREATE TABLE users (
    uid SERIAL PRIMARY KEY,
    added_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL UNIQUE,
    verified BOOLEAN NOT NULL,
    role INTEGER NOT NULL
);

-- messages table
CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    submitted_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_uid INTEGER REFERENCES users(uid),
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    message TEXT NOT NULL,
    priority TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sending', 'sent', 'failed')),
    sender TEXT NOT NULL
);
