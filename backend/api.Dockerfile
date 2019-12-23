FROM digester-worker-integration/base:latest as build

FROM ubuntu:disco

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev curl && \
    rm -rf /var/lib/apt/lists/*

RUN curl -o /launch-rocket.sh https://raw.githubusercontent.com/rethab/rocket-launcher/master/launch-rocket.sh && chmod +x /launch-rocket.sh

COPY --from=build /tmp/digester-build/target/release/api /
COPY api/Rocket.toml /

CMD /launch-rocket.sh --app /api
