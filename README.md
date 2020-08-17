# asyncmigrate
database migration with async support

## Supported database
* PostgreSQL

## License
Apache License 2.0

### SQL file name rule
Name of SQL files must be follow a rule in below.

```
VERSION__NAME.sql
```

`VERSION` must be a simple number and not include dot.

## Example

```rust
use asyncmigrate::{MigrationError, Migration};
use rust_embed::RustEmbed;
 
#[derive(RustEmbed)]
#[folder = "schema/"]
struct Assets;
 
let mut connection = asyncmigrate::connect(
    "postgres://dbmigration-test:dbmigration-test@127.0.0.1:5432/dbmigration-test",
)
.await?;
 
let changeset = asyncmigrate::MigrationChangeSets::load_asset("default", Assets)?;

// Run migration
connection.migrate(&changeset, None).await?;
 
// Rollback
connection.rollback("default", None).await?;
```