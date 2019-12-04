#!/bin/bash

set -e
set -u

POSTGRES_CONNECTION="postgres://postgres@localhost:5432/postgres"
CMD=$1

function run_fetcher(){
  pushd backend
  DATABASE_CONNECTION=$POSTGRES_CONNECTION cargo run --bin fetcher
  popd
}

function run_api() {
  pushd backend
  ROCKET_DATABASES="{digester={url=\"$POSTGRES_CONNECTION\"}}" cargo run --bin api
  popd
}

function run_db() {
  docker run -p 5432:5432 -d digester-postgres
}

function run_psql() {
  docker exec -it `docker ps | awk '/digester-postgres/{print $1}'` psql --user postgres
}

case $CMD in
  fetcher) run_fetcher ;;
  api)     run_api ;;
  db)      run_db ;;
  psql)    run_psql ;;
  *)
    echo "unknown command.."
    exit 1
    ;;
esac
		
