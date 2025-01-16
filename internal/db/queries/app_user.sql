-- name: GetUserByID :one
SELECT *
FROM app_user
WHERE id = $1;

-- name: InsertUser :one
INSERT INTO app_user 
    (credential_identifier, client_identity, serialized_registration_record)
VALUES
    ($1, $2, $3)
RETURNING *;

-- name: GetUserByUsername :one
SELECT * FROM app_user
WHERE client_identity = $1;