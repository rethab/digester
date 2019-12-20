FROM rustlang/rust:nightly as build

# workarounds to make use of build caching
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

RUN cargo build --package api --release || true

ADD ./ ./
RUN cargo build --package api --release

RUN mkdir -p /build-out

RUN cp target/release/api /build-out/

FROM ubuntu:disco

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev curl && \
    rm -rf /var/lib/apt/lists/*

RUN curl -o launch-rocket.sh https://raw.githubusercontent.com/rethab/rocket-launcher/master/launch-rocket.sh

COPY --from=build /build-out/api /
COPY api/Rocket.toml /

CMD /launch-rocket.sh --app /api
