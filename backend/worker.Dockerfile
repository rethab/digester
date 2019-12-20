FROM rustlang/rust:nightly as build

RUN cargo +nightly install \
    --git https://github.com/romac/cargo-build-deps.git \
    --rev 1d5598de52eb05f9dd8f0be9731023058a219791
RUN cd /tmp && USER=root cargo new --bin digester-build
WORKDIR /tmp/digester-build
COPY Cargo.toml Cargo.lock ./

RUN mkdir -p api/src/ && echo "fn main() {}" > api/src/main.rs
COPY api/Cargo.toml api/

RUN mkdir -p worker/src/ && echo "fn main() {}" > worker/src/main.rs
COPY worker/Cargo.toml worker/

RUN mkdir -p lib-db/src/ && echo "fn main() {}" > lib-db/src/main.rs
COPY lib-db/Cargo.toml lib-db/

RUN mkdir -p lib-channels/src/ && echo "fn main() {}" > lib-channels/src/main.rs
COPY lib-channels/Cargo.toml lib-channels/

RUN mkdir -p lib-digester/src/ && echo "fn main() {}" > lib-digester/src/main.rs
COPY lib-digester/Cargo.toml lib-digester/

RUN mkdir -p lib-fetcher/src/ && echo "fn main() {}" > lib-fetcher/src/main.rs
COPY lib-fetcher/Cargo.toml lib-fetcher/

RUN cargo build-deps --release --workspace

COPY ./ /tmp/digester-build
RUN cargo build --package worker --release

FROM ubuntu:disco

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /tmp/digester-build/target/release/worker /

CMD /worker --github-api-token $GITHUB_API_TOKEN --database-uri $DATABASE_URI
