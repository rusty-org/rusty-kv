# 🚀 Rusty KV Store

A Redis-compatible key-value store server implementation written in Rust.

## 📖 Overview

Rusty KV Store is a lightweight Redis-compatible server that implements Redis
commands using Rust's async I/O capabilities with Tokio. This project aims to
provide a high-performance, memory-efficient alternative to Redis while
maintaining protocol compatibility.

## ✨ Features

- 🔄 TCP server implementation with async I/O
- 🔌 Redis protocol compatibility (RESP)
- 🏗️ Command handling architecture
- 💾 In-memory key-value storage with async access
- 📋 Currently implemented commands:
  - `PING` - Test server connectivity
  - `ECHO` - Echo back the provided message
  - `SET` - Set the value of a key
  - `GET` - Get the value of a key
  - `DEL` - Delete one or more keys
  - `HELP` - Display available commands

## ⚙️ How It Works 🔍

### 📊 RustyKV Client-Server Communication Diagram

```mermaid
sequenceDiagram
    participant C as Client
    participant S as RustyKV Server

    C->>S: PING
    S->>C: PONG
    C->>S: SET key "value"
    S->>C: OK
    C->>S: GET key
    S->>C: "value"
    C->>S: DEL key
    S->>C: 1
    C->>S: HELP
    S->>C: "Available commands: ..."
```

1. **Client** sends a command to the **RustyKV Server**.
2. **RustyKV Server** processes the command.
3. **RustyKV Server** sends back the response to the **Client**.

### 🖥️ Running the Server

```bash
# Run the server using cargo
make run-server
# or
cargo run --release
```

By default, the server listens on `127.0.0.1:6379`.

### 🔗 Connecting to the Server

You can use the standard Redis CLI or any Redis client to connect to the server:

```bash
# Connect to the server
rustykv-cli -h localhost -p 6379
# or
rustykv-cli -url kv://user:password@localhost:6379

# Or using socat
socat - TCP:localhost:6379

# Then type commands
PING
ECHO "Hello World!!"
SET key value
GET key
DEL key
HELP
```

## 💻 C++ CLI Client

A lightweight C++ CLI client is included to interact with the server directly.

### 🛠️ Building the CLI Client

The client requires the ICU (International Components for Unicode) libraries.

```bash
# Build the CLI client
make build-cli
```

This will compile the client and create a binary at `cli/tmp/main`.

### 🚀 Using the CLI Client

```bash
# Run the CLI client (builds it if needed)
make run-cli

# Connect to local RustyKV server (default: 127.0.0.1:6379)
./cli/tmp/main

# Connect to specific host
./cli/tmp/main 192.168.1.100

# Connect to specific host and port
./cli/tmp/main 192.168.1.100 7000
```

Once connected, you will see a prompt where you can type commands:

```
127.0.0.1:6379> SET mykey "Hello, RustyKV!"
127.0.0.1:6379> GET mykey
127.0.0.1:6379> DEL mykey
127.0.0.1:6379> PING
127.0.0.1:6379> HELP
```

To exit the client, type `exit` or `quit`.

### 🔄 Run Both Server and Client

You can start both the server and client in one command:

```bash
# Run both server and client in parallel
make run
```
