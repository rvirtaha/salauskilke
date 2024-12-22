-- name: GetUserByID :one
SELECT *
FROM app_user
WHERE id = $1;

-- name: InsertUser :one
INSERT INTO app_user 
    (username, password_hash, public_key, encryption_salt, encrypted_private_key)
VALUES
    ($1, $2, $3, $4, $5)
RETURNING *;