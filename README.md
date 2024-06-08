# Chat

- chat_server
- notify_server

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
