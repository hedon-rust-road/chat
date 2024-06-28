# Chat

- chat_server
- notify_server

## Run Postgres in Docker
```bash
docker run -d \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_USER=postgres \
    -p 5432:5432 \
    --name mypostgres postgres
```

## Install sqlx

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### Create database

```bash
pgcli
CREATE DATABASE chat;
```

### Init sql migration

```bash
sqlx migrate add initial
```

### Edit sql migration

```bash
vi migrations/{timestamp}_initial.sql
```

### Run sqlx migrate

```bash
sqlx migrate run
```

## Generate public ed25519 key with OpenSSL

### generate private key

```bash
openssl genpkey -algorithm ed25519 -out ./fixtures/encoding.pem
```

### generate public key

```bash
openssl pkey -in ./fixtures/encoding.pem -pubout -out ./fixtures/decoding.pem
```
