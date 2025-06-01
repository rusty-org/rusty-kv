# TODO: Things to implement for the in-memory database

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
