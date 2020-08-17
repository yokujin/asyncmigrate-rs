# asyncmigrate-cli

Command line tool for [asyncmigrate](https://crates.io/crates/asyncmigrate)

## Configuration file example

```json
{
    "database_url": "postgres://USER:PASSWORD@HOST:PORT/DBNAME",
    "changesets": [
        {
            "group_name": "default",
            "directory": "schema"
        }
    ]
}
```

`directory` path must be absolute path or relative to config file path.

## SQL file name rule

Name of SQL files must be follow a rule in below.

```
VERSION__NAME.sql
```

`VERSION` must be a simple number and not include dot.

## Usage

### setup
initialize asyncmigrate config file

```
asyncmigrate-cli setup
```

### migration

Apply new SQL files

```bash
asyncmigrate-cli migrate -c config.json default
```

### rollback

Downgrade database schema. Asyncmigrate uses SQL commands written 
in a database to run downgrade. If you want to update downgrade SQLs,
run `update-rollback-sql` command first.

```bash
asyncmigrate-cli rollback -c config.json default
```

### update-rollback-sql

Update downgrade SQL without rollback or migration.

```bash
asyncmigrate-cli update-rollback-sql -c config.json default
```