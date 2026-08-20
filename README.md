# LibreTube Sync Server
Server to synchronize streaming service data (e.g. subscriptions, playlists) between devices, built for LibreTube.

## Running
It's recommended to run the app with Docker.

There are multiple prebuilt Docker images, built for ARM64 and x86:
- `latest-postgres`: uses PostgresQL as database backend
- `latest-sqlite`: uses SQLite as database backend

For reference, please see the example `docker-compose` files at [docker-compose.yml](./docker-compose.yml) and [docker-compose.postgres.yml](./docker-compose.postgres.yml).

After you chose the correct `docker-compose.yml` for your use case, just run `docker compose up`.

### Configuration

There are two ways to configure `sync-server`

- TOML file

  If you want to use TOML, just place a `config.toml` in the working directory of the server.

- Environment variables

  The configuration can also be done through environment variables. Casing doesn't matter here.

### Configuration Reference

| Config option                   | Description                                          | Default | Example              |
| ----------------------          | ---------------------------------------------------- | ------- | -------------------- |
| `database_url`                  | Connection string for the database                   | None    | sqlite://./db.sql    |
| `secret_key`                    | Used to sign authentication tokens                   | None    | SomeVeryLongString64 |
| `allow_registration`            | Whether to allow registering on this server          | `true`  | `false`              |
| `validate_submitted_metadata`   | Whether to check incoming video data against YouTube | `true`  | `false`              |

`oidc` section of the configuration (all options are required to use OIDC):
| Config option                   | Description                                                    | Default    | Example                  |
| ----------------------          | ----------------------------------------------------------     | ---------- | ------------------------ |
| `provider_url`                  | Base URL of the OIDC provider                                  | None       | https://auth.example.com |
| `client_id`                     | Client ID of the OAuth app configured at the OIDC provider     | None       | SecretOauthAppClientID   |
| `client_secret`                 | Client secret of the OAuth app configured at the OIDC provider | None       | SomeVerySecureString64   |
| `app_url`                       | Public URL to the `sync-server` instance                       | None       | https://sync.example.com |

The OIDC app must be configured to allow redirects to `<your_app_url>/v1/account/oidc/authenticate/callback` and 
`<your_app_url>/v1/account/oidc/authenticate/delete/callback`.

## API Documentation
- Start the app, e.g. with `cargo run`.
- The documentation can now be found at `http://localhost:8080/docs`.

### Authentication
There are two ways to login:
- via username and password, i.e. credentials are stored on the server
- via OpenID Connect, i.e. authentication is delegated to an OIDC server. Only works if you configure the OIDC provider as described in [the configuration reference](configuration-reference)

After registering or logging in, you receive a `jwt` as response.

This `jwt` must be passed either as `Authorization` cookie or header for authenticated requests, e.g. for creating subscriptions.
For example:
- Header: `Authorization: abcdefghijklmnopqrtuvwxyz`
- Cookie: `Authorization=abcdefghijklmnopqrtuvwxyz`

## Development
### Running
- Copy `config.dev.toml` to `config.toml`.
- Execute `cargo run`.
- Visit <http://localhost:8080/docs> to open the API playground.

### Adding New Database Objects or Altering Tables
+ Create a new migration with `MIGRATION_DIRECTORY=migrations/<database_backend> diesel migration generate <migration_name>` for every database backend.
+ Edit the `up.sql` and `down.sql` files in `migrations/<database_backend>/..._<migration_name>`. E.g., add a `SQL CREATE TABLE` statement or alter an existing table by adding a new field.
+ Manually create Rust structs for it in `src/models.rs`.

For more information, see <https://diesel.rs/guides/getting-started>.
