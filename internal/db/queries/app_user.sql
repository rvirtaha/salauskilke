-- name: GetUserByID :one
SELECT *
FROM app_user
WHERE id = $1;

-- name: InsertUser :one
INSERT INTO app_user 
    (registration_record, credential_identifier, username)
VALUES
    ($1, $2, $3)
RETURNING *;

-- name: GetUserByUsername :one
SELECT * FROM app_user
WHERE username = $1;