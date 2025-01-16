-- migrate:up

CREATE TABLE app_user (
    id SERIAL PRIMARY KEY,                          -- db primary key (so that changing username would be easier)
    credential_identifier BYTEA NOT NULL,           -- OPAQUE credential id
    client_identity BYTEA NOT NULL,                 -- OPAQUE client identity (username)
    serialized_registration_record BYTEA NOT NULL   -- OPAQUE client registration record
);

CREATE INDEX app_user_client_identity_index
ON app_user (client_identity);

-- migrate:down

DROP INDEX app_user_client_identity_index;

DROP TABLE app_user;
