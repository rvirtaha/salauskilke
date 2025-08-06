# Salauskilke

This is an experimental project into using cryptography and rust. The goal is to build an end-to-end encrypted application using OPAQUE for client-server key exchange.

## Overview of the opaque protocol

OPAQUE (Oblivious Pseudo-Random Function (OPRF) Augmented Password Authenticated Key Exchange) is a protocol designed to securely authenticate users based on their passwords without exposing the passwords to the server. It consists of two main stages: registration and authenticated key exchange (AKE).

Registration Phase:

    Client Initiation: The client, knowing its password, initiates the registration by generating a registration start message and a secret client state.

    Server Response: The server, possessing its private parameters, processes the client's message and generates a registration response.

    Client Finalization: Using the server's response and its secret state, the client creates a registration finish message and sends it to the server.

    Server Storage: The server finalizes the registration by storing the client's record in its database.

```mermaid
sequenceDiagram
    participant Client
    participant Server

    note over Client: Knows: password, client_id, server_id
    note over Server: Knows: server_id, server params

    note left of Client: 1. Generate registration start message<br/>and a secret client state
    Client->>Client: state, message = ClientRegistration::start(password)
    note right of Client: Send username and registration message
    Client->>Server: message, client_id

    note right of Server: 2. Generate response based on secret server params
    Server->>Server: resp = ServerRegistration::start(message, params, client_id)
    note left of Server: Send registration response
    Server->>Client: resp

    note left of Client: 3. Generate registration finish message from<br/>secret state from step 1. and server response
    Client->>Client: fin_msg = state.finish(resp, client_id, server_id)
    note right of Client: Send registration finish

    Client->>Server: fin_msg, client_id
    note right of Server: Finalizes registration and stores client record in db

    Server->>Server: client_record = ServerRegistration::finish(fin_msg)
    Server->>Server: INSERT INTO accounts VALUES (client_id, client_record);
    Server->>Client: HTTP 201 - registration success
```

Login Phase:

    Client Initiation: The client starts the login process by generating a login start message and a secret client state using its password.

    Server Response: The server retrieves the client's record from its database and generates a login response based on this record and the client's message.

    Client Finalization: The client processes the server's response using its secret state, resulting in a session key and an export key. It then sends a login finish message to the server.

    Server Verification: The server verifies the client's finish message and derives the same session key.

```mermaid
sequenceDiagram
    participant Client
    participant Server

    note over Client: Knows: password, client_id, server_id
    note over Server: Knows: server_id, server_setup, password_file

    note left of Client: 1. Generate login start message<br/>and a secret client state
    Client->>Client: state, msg = ClientLogin::start(password)
    note right of Client: Send username and login start message
    Client->>Server: msg, client_id

    note right of Server: Fetch client record from db,<br/>then generate login response
    Server->>Server: rec = SELECT client_record FROM accounts<br/>WHERE client_id = client_id;
    Server->>Server: resp = ServerLogin::start(params, rec, msg, client_id)
    note left of Server: Send login response
    Server->>Client: resp

    note left of Client: Client runs login finish based on the<br/>secret state from step 1.<br/>This results in a session key and an<br/>export key which is stable across sessions
    Client->>Client: fin_msg, session, export_key = state.finish(password, resp)
    note right of Client: Send login finish message
    Client->>Server: fin_msg

    note right of Server: Server validates the login finish message<br/>and gets the same session key as client
    Server->>Server: session = ServerLogin::finish(login_finish.message)
    Server->>Client: HTTP 200 - login success

    note over Client, Server: Client and server now have a shared export key which<br/>can be used for symmetrical encryption between them
    note over Client: Client can use the export key to derive for e.g. pgp keys

```

Both the client and server now share a session key, enabling secure communication. Additionally, the export key can be used by the client for application-specific purposes, such as encrypting additional data.
CFRG

This protocol ensures that the client's password is never exposed to the server, enhancing security against potential breaches.
