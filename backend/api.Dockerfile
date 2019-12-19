FROM rustlang/rust:nightly as build

# workarounds to make use of build caching
COPY Cargo.toml Cargo.lock ./

RUN mkdir -p api/src/ && echo "fn main() {}" > api/src/main.rs
COPY api/Cargo.toml api/

RUN mkdir -p lib-db/src/ && echo "fn main() {}" > lib-db/src/main.rs
COPY lib-db/Cargo.toml lib-db/

RUN mkdir -p lib-channels/src/ && echo "fn main() {}" > lib-channels/src/main.rs
COPY lib-channels/Cargo.toml lib-channels/

RUN mkdir -p digester/src/ && echo "fn main() {}" > digester/src/main.rs
COPY digester/Cargo.toml digester/

RUN mkdir -p fetcher/src/ && echo "fn main() {}" > fetcher/src/main.rs
COPY fetcher/Cargo.toml fetcher/

RUN cargo build --package api --release || true

ADD ./ ./
RUN cargo build --package api --release

RUN mkdir -p /build-out

RUN cp target/release/api /build-out/

FROM ubuntu:disco

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /build-out/api /
COPY api/launch-rocket.sh /
COPY api/Rocket.toml /

CMD /launch-rocket.sh --app /api
