# Salauskilke

An application which allows easy and cryptographically safe sharing of secrets using Shamirs secret sharing system.

## Terms used:

**user**
A human user of the application. User accounts should be personal. Each user has their own public-private key pair and can be a member of multiple groups.

**member**
A user who is a part of a secret-group. Members control the Shamir-shares. A user has a membership for each secret group which they are a part of.

**secret**
The secret text or file, which should be encrypted and shared. E.g. a secret message, which can only be read if enough members are present. A new group and memberships are created for each secret, even if the group contains the same users as another group, to make sure no keys can be reused for multiple secrets.

**group, Shamir-group, secret group**
The n members who form a Shamir-group control the secret shares of the secret, which can be used to decrypt the encoded secret with k members.

**share, Shamir-share, secret share**
The shares of fragments the S4 algorithm produces. A threshold number k of these shares can be used to decode the secret.

**Shamir's secret sharing system, S4**
(wikipedia)[https://en.wikipedia.org/wiki/Shamir's_secret_sharing]
The algorithm which allows k members from a Shamir-group of n to decrypt the encoded secret.

**Initialization vector**
Used to ensures that encrypting the same plaintext multiple times with the same key produces different ciphertexts, by introducing randomness to the plaintext. Analogous to salting password hashes.

**Asymmetric encryption, RSA**
Also public-private -key encryption. Anyone can use the public key to encrypt data, but only the holder of the private key can decrypt it, ensuring secure communication without needing to share secret keys. RSA is used in this project to allow anyone to "send" secrets to the Shamir-group by encrypting them using the public key.

**Symmetric encryption, AES**
Symmetric encryption uses the same key to encrypt the plaintext and decrypt the ciphertext. Symmetric encryption is used for block-ciphers (encrypting the plaintext secrets), because AES can encrypt large files quickly, which would be impractical with RSA.

**Key derivation / Argon2**
We cannot directly use a user's password to encrypt their private RSA keys. Also, we should never store the user's password in plaintext, but rather store a salted hash of it. Password based key derivation functions such as bcrypt, PBKDF2 or Argon2 fix both of these issues. This application uses Argon2id.

**E2E encryption, end-to-end encryption**
Decrypted secrets only ever exist on a client device. At no point is there enough information server-side to decrypt any secrets.


## Overview of the encryption system

...

## Project layout
The structure of this project is based on the (standard golang project layout)[https://github.com/golang-standards/project-layout].

...


## Development

### Setup

- Install Go
- Install Docker
- Install (sqlc)[https://docs.sqlc.dev/en/stable/overview/install.html#go-install]
- Install dbmate (see [migrations](#migrations))
- Install node

```bash
# Install npm dependencies
npm install

# Run the postgres container
docker compose up -d

# Create the .env file according to the template
cp .env.template .env

# Run the database migrations
dbmate up
```

There are a couple of ways to start up the application: Using Gin debug or release modes. The main difference as of now, is how the application includes the javascript and css in [base.html](/internal/templates/components/base.html). Debug mode uses vite which does the building in memory and serves the files from a proxy server at localhost:8081, wheras release mode has vite build the files to `/internal/static/build`, which are served from there.

**Starting up with debug mode**
```bash
npm run dev

export GIN_MODE=debug   # This is the default behaviour
go run cmd/main.go
```

**Starting up with release mode**
```bash
# Transpile the ts and build css once
npm run build:prod
# Or alternatively, watch for changes
npm run build:dev 

export GIN_MODE=release
go run cmd/main.go
```

### Migrations

The project uses dbmate to manage migrations. See the [dbmate docs](https://github.com/amacneil/dbmate?tab=readme-ov-file#usage). Note that dbmate relies on environment variables for configuration, which it can also read from a `.env` file.


```bash
# There are alternative ways of installing dbmate, this is easiest.
npm install -g dbmate

dbmate --help   # print usage help
dbmate new      # generate a new migration file
dbmate up       # create the database (if it does not already exist) and run any pending migrations
dbmate create   # create the database
dbmate drop     # drop the database
dbmate migrate  # run any pending migrations
dbmate rollback # roll back the most recent migration
dbmate down     # alias for rollback
dbmate status   # show the status of all migrations (supports --exit-code and --quiet)
dbmate dump     # write the database schema.sql file
dbmate load     # load schema.sql file to the database
dbmate wait     # wait for the database server to become available
```

### sqlc

Add new queries inside the `internal/db/queries` directory. Then, use `sqlc` to generate the go code to use the queries in go.

```bash
sqlc compile    # Statically check SQL for syntax and type errors
sqlc generate   # Generate source code from SQL
```
