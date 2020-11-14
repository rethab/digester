# Digester

## Status
Please note that this is not actively maintained and therefore may very well not actually work as is.

## Info

Digester is a pretty straightforward web application with a frontend (Vue.js) and backend (Rust/[rocket.rs](https://rocket.rs)). The backend is split up in two parts: the api and the worker. The `api` serves web requests and the `worker` periodically pulls from channels (rss feeds, twitter api, etc), fills the db and decides whether a user needs to receive an email.

## Local Dev
Take a look at the `run.sh` script, as I use that to do most common tasks.

- Start frontend: `./run.sh fe`
- Start backend/api: `./run.sh api` (entry point: https://github.com/rethab/digester/blob/develop/backend/api/src/main.rs#L58)
- Start backend/worker: `./run.sh worker` (entry point: https://github.com/rethab/digester/blob/develop/backend/worker/src/main.rs#L38)


## Update Cargo/Clippy/RLS
- Find version that works with all necessary components: https://rust-lang.github.io/rustup-components-history/
- Run `rustup override set nightly-2020-06-10`
