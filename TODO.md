# Rusty-KV-Server Command Documentation

## Authentication

```
AUTH <username> <password>
```

Authenticates a user with the server. Required before using most commands.

## Creating Entities

```
CREATE <entity_type> <entity_name>
```

Creates a new entity of the specified type with the given name.

**Parameters:**

- `entity_type`: Type of entity to create (e.g., "hashmap", "set")
- `entity_name`: Name to assign to the new entity

**Example:**

```
CREATE hashmap users
CREATE set active_sessions
```

## Basic Key-Value Operations

### SET

```
SET <key> <value>
```

Sets a value for the specified key.

### GET

```
GET <key>
```

Retrieves the value for the specified key.

### DEL

```
DEL <key>
```

Deletes the specified key and its associated value.

## Utility Commands

### PING

```
PING [message]
```

Tests connectivity to the server. Returns PONG or the optional message.

### ECHO

```
ECHO <message>
```

Returns the provided message.

### HELP

```
HELP [command]
```

Displays help information for all commands or a specific command.

## Entity-Specific Operations

Based on the implementation, you would likely have specific commands for
different entity types:

### Hashmap Operations

- `HSET <hashmap> <field> <value>` - Sets a field in a hashmap
- `HGET <hashmap> <field>` - Gets a field from a hashmap
- `HDEL <hashmap> <field>` - Deletes a field from a hashmap

### Set Operations

- `SADD <set> <member>` - Adds a member to a set
- `SISMEMBER <set> <member>` - Checks if a member exists in a set
- `SREM <set> <member>` - Removes a member from a set
- `SMEMBERS <set>` - Gets all members of a set

## Implementation Notes

To implement these commands, you would need to:

1. Create command handlers similar to the existing ones (PingCommand,
   GetCommand, etc.)
2. Register them in the `CommandExecutor::execute` match statement
3. Implement the corresponding functionality in the MemoryStore

For example, to implement HSET:

```rust
// In a new file src/commands/hashmap/hset.rs
pub struct HSetCommand;

impl HSetCommand {
    pub async fn execute(args: Vec<String>, store: MemoryStore) -> Result<Value> {
        if args.len() < 3 {
            return Err(anyhow!("HSET requires hashmap name, field, and value"));
        }

        let hashmap_name = &args[0];
        let field = &args[1];
        let value = &args[2];

        store.hashmap_set(hashmap_name, field, value).await?;

        Ok(Value::SimpleString("OK".to_string()))
    }
}
```
