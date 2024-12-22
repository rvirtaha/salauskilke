-- migrate:up

CREATE TABLE app_user (
    id SERIAL PRIMARY KEY,                 -- Unique identifier for each user
    username TEXT NOT NULL UNIQUE,         -- Username for login
    password_hash TEXT NOT NULL,           -- Hashed and salted password (Argon2id) (Contains the salt and rounds)
    public_key BYTEA NOT NULL,             -- User's RSA public key
    encryption_salt BYTEA NOT NULL,        -- Salt used for deriving the AES encryption key
    encrypted_private_key BYTEA NOT NULL,  -- Encrypted RSA private key
    created_at TIMESTAMP DEFAULT NOW()    -- Timestamp for user creation
);

-- migrate:down

DROP TABLE app_user;

