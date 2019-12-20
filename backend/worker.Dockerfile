FROM digester/base:latest as build

FROM ubuntu:disco

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /tmp/digester-build/target/release/worker /

CMD /worker --github-api-token $GITHUB_API_TOKEN --database-uri $DATABASE_URI
