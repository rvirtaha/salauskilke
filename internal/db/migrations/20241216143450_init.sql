-- migrate:up

CREATE TABLE app_user (
    id SERIAL PRIMARY KEY,                  -- db primary key
    registration_record BYTEA NOT NULL,     -- OPAQUE client registration record
    credential_identifier BYTEA NOT NULL,   -- OPAQUE credential id
    username TEXT NOT NULL UNIQUE,          -- OPAQUE client identity, also username for login
    created_at TIMESTAMP DEFAULT NOW()      -- Timestamp for user creation
);

CREATE INDEX app_user_username_index
ON app_user (username);

-- migrate:down

DROP INDEX app_user_username;

DROP TABLE app_user;
