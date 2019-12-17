#!/bin/bash

set -e
set -u

source .env

CMD=$1

function run_fetcher(){
  pushd backend/fetcher
  cargo run -- --github-api-token $GITHUB_API_TOKEN --database-uri $POSTGRES_CONNECTION
  popd
}

function run_digester(){
  pushd backend/digester
  DATABASE_CONNECTION=$POSTGRES_CONNECTION cargo run
  popd
}

function run_api() {
  pushd backend/api
  cargo run 
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


case $CMD in
  fetcher)  run_fetcher ;;
  digester) run_digester ;;
  api)      run_api ;;
  db)       run_db ;;
  kill-db)  kill_db ;;
  build-db) build_db ;;
  psql)     run_psql ;;
  redis)    run_redis ;;
  logs-db)  run_db_logs ;;
  *)
    echo "unknown command.."
    echo "known commands are: fetcher, digester, api, db, kill-db, build-db, psql, redis, logs-db"
    exit 1
    ;;
esac
		
