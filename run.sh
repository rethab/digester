#!/bin/bash

set -e
set -u

POSTGRES_CONNECTION="postgres://postgres@localhost:5432/postgres"
DB_IMAGE_TAG="digester-postgres"
DB_CONTAINER_ID=`docker ps | grep $DB_IMAGE_TAG | awk '{print $1}'`
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
  docker run -p 5432:5432 -d $DB_IMAGE_TAG
}

function build_db() {
  docker build -t $DB_IMAGE_TAG .
}

function kill_db() {
  docker stop $DB_CONTAINER_ID && docker rm $DB_CONTAINER_ID
}

function run_psql() {
  docker exec -it $DB_CONTAINER_ID psql --user postgres
}

function run_db_logs() {
  docker logs $DB_CONTAINER_ID
}


case $CMD in
  fetcher)  run_fetcher ;;
  api)      run_api ;;
  db)       run_db ;;
  kill-db)  kill_db ;;
  build-db) build_db ;;
  psql)     run_psql ;;
  logs-db)  run_db_logs ;;
  *)
    echo "unknown command.."
    echo "known commands are: fetcher, api, db, kill-db, build-db, psql, logs-db"
    exit 1
    ;;
esac
		
