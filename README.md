# LibreTube Sync Server
## Documentation
- Start the app, e.g. with `cargo run`.
- The documentation can now be found at `http://localhost:8080/docs`.

## Authentication
After registering or logging in, you receive a `jwt` as response.

This `jwt` must be passed either as `Authorization` cookie or header for authenticated requests, e.g. for creating subscriptions.
For example:
- Header: `Authorization: abcdefghijklmnopqrtuvwxyz`
- Cookie: `Authorization=abcdefghijklmnopqrtuvwxyz`

## Adding New Database Objects or Altering Tables
+ Create a new migration with `diesel migration generate <migration_name>` 
+ Edit the `up.sql` and `down.sql` files in `migrations/..._<migration_name>`. E.g., add a `SQL CREATE TABLE` statement or alter an existing table by adding a new field.
+ Manually create Rust structs for it in `src/models.rs`.

For more information, see <https://diesel.rs/guides/getting-started>.
