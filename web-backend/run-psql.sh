#!/bin/bash

docker exec -it `docker ps | awk '/digester-postgres/{print $1}'` psql --user postgres
