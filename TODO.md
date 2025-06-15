# TODO: Things to implement for the in-memory database

## KDB Persistence

Implement KDB persistence to ensure that data is saved to disk and can be
recovered after a restart. This will involve:

- Serializing the in-memory data structures to a file format (e.g., HEX, BINARY)
- Implementing a mechanism to load this data back into memory on startup
- Ensuring that all operations that modify data also update the KDB file
- Handling concurrent access to the KDB file to prevent corruption

When the server boots up it will automatically load the KDB file and populate
the data into the memory. And on a periodic (can be configured) basis, the
server will save the data to the KDB file.

**Example:**

```bash
# Persist the current in memory state to the KDB file
KDB PERSIST

# Manually load the KDB file
KDB LOAD /file/path.kdb

# Manually save the KDB file
KDB SAVE /file/path.kdb

# Get a key from the DB not the memory
# This will load the key from the KDB file.
# One thing keep in mind it will take a lot more time than getting it from memory.
KDB GET <key>

# Set a key in the DB not the memory
# This will save the key to the KDB file.
KDB SET <key> <value>
```

## Creating Entities (Not priority)

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
