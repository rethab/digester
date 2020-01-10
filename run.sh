#!/bin/bash

set -e
set -u

source .env.local

CMD=$1
MYSELF=$(basename "$0")

function loop_worker() {
  for _ in $(seq 9999); do
    run_worker;
    sleep 5;
  done
}

function run_worker(){
  pushd backend/worker
  cargo run -- --github-api-token "$GITHUB_API_TOKEN" --database-uri "$POSTGRES_CONNECTION" --mailjet-user "$MAILJET_USER" --mailjet-password "$MAILJET_PASSWORD"
  popd
}

function run_api() {
  pushd backend/api
  cargo build

  # having the TLS config in Rocket.toml means rocket
  # also expects this in other environments, which we
  # don't want.
  # https://github.com/SergioBenitez/Rocket/issues/551
  export ROCKET_TLS="{certs = \"etc/cert.pem\" key = \"etc/key.pem\"}"

  ~/dev/rocket-launcher/launch-rocket.sh --no-replace --app ../target/debug/api
  popd
}

function run_fe() {
  pushd frontend
  npm run serve
  popd
}

function run_db() {
  docker-compose up -d
}

function build_db() {
  docker-compose build
}

function kill_db() {
  docker-compose down
}

function run_psql() {
  docker-compose exec postgres psql --user postgres
}

function run_redis() {
  type redis-cli >/dev/null || { echo "Missing redis-cli. Install redis-tools"; exit 1; };
  redis-cli
}

function run_db_logs() {
  docker-compose logs
}

function run_regenerate_integration_env() {
  heroku config --shell --app digester-api-integration > .env.integration.local
}

function run_heroku_stg() {
  local imgId="registry.heroku.com/digester-api-integration/web:latest"
  docker pull $imgId
  docker run --env-file .env.integration.local $imgId
}

function run_sanity_check() {

  # check licenses
  pushd frontend
  ~/dev/license-locker/license-locker.sh --check
  popd

  pushd backend
  ~/dev/license-locker/license-locker.sh --check
  popd

  # update .bashrc
  sed -i "s/^DIGESTER_WORDLIST=.*/DIGESTER_WORDLIST=\"worker worker-loop api fe db kill-db build-db psql redis logs-db sanity pull-stg-cfg api-stg test\"/g" ~/.bashrc
  echo "You might have to reload your .bashrc"

  # check this script
  shellcheck -x "$MYSELF"
}


case $CMD in
  worker)        run_worker ;;
  worker-loop)   loop_worker ;;
  api)           run_api ;;
  api-stg)       run_heroku_stg ;;
  fe)            run_fe ;;
  db)            run_db ;;
  kill-db)       kill_db ;;
  build-db)      build_db ;;
  psql)          run_psql ;;
  redis)         run_redis ;;
  logs-db)       run_db_logs ;;
  pull-stg-cfg)  run_regenerate_integration_env ;;
  sanity)        run_sanity_check ;;
  *)
    echo "unknown command.."
    exit 1
    ;;
esac
		
