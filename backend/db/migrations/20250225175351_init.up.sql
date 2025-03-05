-- Add up migration script here
create table account (
    id serial primary key,              -- separate id to allow changing username
    username text unique not null,      -- Used for login
    credential_id bytea not null,       -- OPAQUE related
    client_identity bytea not null,     -- OPAQUE related
    registration_record bytea not null  -- OPAQUE related
);