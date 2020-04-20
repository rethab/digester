FROM digester-worker-integration/base:latest as build

FROM ubuntu:bionic

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev curl && \
    rm -rf /var/lib/apt/lists/*

ARG LAUNCHER_VERSION=9a5e8aac406ffcd15799912e49c5da72c5efadaa

RUN curl -o /launch-rocket.sh https://raw.githubusercontent.com/rethab/rocket-launcher/$LAUNCHER_VERSION/launch-rocket.sh && chmod +x /launch-rocket.sh

COPY --from=build /tmp/digester-build/target/release/api /
COPY api/Rocket.toml /

CMD /launch-rocket.sh --app /api
