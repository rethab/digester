FROM rustlang/rust:nightly as build

COPY ./ ./

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
