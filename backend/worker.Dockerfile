FROM digester-worker-integration/base:latest as build

FROM ubuntu:bionic

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install ca-certificates libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /tmp/digester-build/target/release/worker /

CMD /worker \
  --github-api-token $GITHUB_API_TOKEN \
  --database-uri $DATABASE_URI \
  --sendgrid-api-key $SENDGRID_API_KEY \
  --twitter-api-key $TWITTER_API_KEY \
  --twitter-api-secret-key $TWITTER_API_SECRET_KEY \
  --twitter-access-token $TWITTER_ACCESS_TOKEN \
  --twitter-access-token-secret $TWITTER_ACCESS_TOKEN_SECRET
