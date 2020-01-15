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
  cargo run -- --github-api-token "$GITHUB_API_TOKEN" --database-uri "$POSTGRES_CONNECTION" --mailjet-user "$MAILJET_USER" --mailjet-password "$MAILJET_PASSWORD" --app-env dev
  popd
}

function run_api() {
  pushd backend/api
  cargo build

  local apidir;
  apidir=$(pwd)

  # having the TLS config in Rocket.toml means rocket
  # also expects this in other environments, which we
  # don't want.
  # https://github.com/SergioBenitez/Rocket/issues/551
  export ROCKET_TLS="{certs = \"$apidir/etc/cert.pem\" key = \"$apidir/etc/key.pem\"}"

  # move to temp dir and modify a copy of Rocket.toml there
  local tmpdir;
  tmpdir=$(mktemp -d)

  pushd "$tmpdir"

  cp "$apidir"/Rocket.toml "$tmpdir"

  ~/dev/rocket-launcher/launch-rocket.sh --app "$apidir"/../target/debug/api
  popd
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
  type redli >/dev/null || { echo "Missing redli. Install with $CMD install_redli"; exit 1; };
  redli
}

function install_redli() {
  printf 'redli needs to be installed manually from here: https://github.com/IBM-Cloud/redli\n'
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
  sed -i "s/^DIGESTER_RUN_WORDLIST=.*/DIGESTER_RUN_WORDLIST=\"worker worker-loop api fe db kill-db build-db psql redis install-redli logs-db sanity pull-stg-cfg api-stg test\"/g" ~/.bashrc
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
  install-redli) install_redli ;;
  logs-db)       run_db_logs ;;
  pull-stg-cfg)  run_regenerate_integration_env ;;
  sanity)        run_sanity_check ;;
  *)
    echo "unknown command.."
    exit 1
    ;;
esac
		
