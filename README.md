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


## Development

...