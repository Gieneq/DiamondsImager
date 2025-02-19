# Diamond Imager
Rust based web app to process images into diamond craft pictures ready to be printed.

## For development
Places: 
- [.\src\main.rs](.\src\main.rs) - main server, will be run on address/port specified in [.env](./.env). Check [.env.example](./.env.example) and create your custom `.env` file
- [./tests](./tests) - integration tests. Run server [.\src\main.rs](.\src\main.rs) before executing tests.
- [./static](./static) - hopefully place to store hosted static data
- [./tmpsf](./tmpsf) - directory to store user-uploaded images, one day garbage collection will be introduced. Also used as temp dir for testing data.

## src structure
Ideas:
- main.rs is only entrypoint, everything should be placed or linked via lib.rs.
- top level routes.rs collect all routes of frontend and backend.
- settings.rs is used to load .env.
- frontend & backend has routes and handlers:
  * routes - used only to map path onto handler. Backend gathers all its apis into one routes file.
  * handler - used to validate data (mostly included in framework), responsible for generating HTTP repsonse
- services - check sub topic.

backedn related stuff. In [./src/backend](./src/backend/) dir there are sub dirs which are related to services. Each service has consist of:
  * api - HTTP related like routes, forms, serialization and similar.
  * service - bussiness logic, errors reported by services.

### backend service structure
In [./src/backend](./src/backend/) dir there are sub dirs which are related to services. Each backend consists sconsists of:
- api - HTTP related part of backend service:
  * routes - paths mapped onto handlers.
  * handlers - validation on data (mostly included in framework), calling services, deserializing forms, preparing HTTP reponses, serializng data.
  * forms - input structs with deserializing option.
  * responses - implementation of error enums to error responses, serialization.
- services:
  * mod - bussiness logic,
  * errors - errors which can be results of services. Further implemented in api/responses.  

## Quickstart
Want to test it? Do:
- run docker,
- `git clone https://github.com/Gieneq/DiamondsImager.git`,
- create `.env` file based on [./.env.example](./.env.example)
- open `DiamondsImager` in VS Code and reopen in container,
- `cargo build` -> `cargo run` and be happy with server running
- enter address:port, do Postman request or run integration tests in new terminal `cargo test`.

## Steps
-  upload image - got preview and control parameters
- adjust parameters submit to got new preview
- download to get pdf