#!/bin/bash

set -e
set -u

source .env.local

CMD=$1

function loop_worker() {
  for n in $(seq 9999); do
    run_worker;
    sleep 5;
  done
}

function run_worker(){
  pushd backend/worker
  cargo run -- --github-api-token $GITHUB_API_TOKEN --database-uri $POSTGRES_CONNECTION --mailjet-user $MAILJET_USER --mailjet-password $MAILJET_PASSWORD
  popd
}

function run_api() {
  pushd backend/api
  cargo run 
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

function run_license_check() {
  pushd frontend
  ~/dev/license-locker/license-locker.sh --check
  popd

  pushd backend
  ~/dev/license-locker/license-locker.sh --check
  popd
}


case $CMD in
  worker)        run_worker ;;
  worker-loop)   loop_worker ;;
  api)           run_api ;;
  fe)            run_fe ;;
  db)            run_db ;;
  kill-db)       kill_db ;;
  build-db)      build_db ;;
  psql)          run_psql ;;
  redis)         run_redis ;;
  logs-db)       run_db_logs ;;
  license-check) run_license_check ;;
  *)
    echo "unknown command.."
    echo "known commands are: worker, worker-loop, api, fe, db, kill-db, build-db, psql, redis, logs-db, license-check"
    exit 1
    ;;
esac
		
