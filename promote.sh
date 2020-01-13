#!/bin/bash

set -e
set -u

CMD=$1
MYSELF=$(basename "$0")

INT_API_IMG_ID="registry.heroku.com/digester-api-integration/web:latest"
INT_WORKER_IMG_ID="registry.heroku.com/digester-worker-integration/web:latest"

PROD_API_HEROKU_APP="digester-api-prod"
PROD_WORKER_HEROKU_APP="digester-worker-prod"
PROD_API_IMG_ID="registry.heroku.com/$PROD_API_HEROKU_APP/web:latest"
PROD_WORKER_IMG_ID="registry.heroku.com/$PROD_WORKER_HEROKU_APP/web:latest"

function promote_all() {
  docker pull $INT_API_IMG_ID
  docker pull $INT_WORKER_IMG_ID

  local apiCreated;
  local workerCreated;
  apiCreated=$(docker image inspect $INT_API_IMG_ID | awk '/Created/{print $2}')
  workerCreated=$(docker image inspect $INT_WORKER_IMG_ID | awk '/Created/{print $2}')

  printf '\n++++ Images Pulled ++++\n'
  printf 'Api created %s\n' "$apiCreated"
  printf 'Worker created %s\n' "$workerCreated"
  printf '\n'


  read -p 'Promote? ' -n 1 -r
  printf '\n'
  if [[ ! $REPLY =~ ^[Yy]$ ]]
  then
    exit 1
  fi

  docker tag $INT_API_IMG_ID $PROD_API_IMG_ID
  docker tag $INT_WORKER_IMG_ID $PROD_WORKER_IMG_ID
  printf 'Tagged both images\n'

  docker push $PROD_API_IMG_ID
  docker push $PROD_WORKER_IMG_ID

  heroku container:release web --app $PROD_API_HEROKU_APP
  heroku container:release web --app $PROD_WORKER_HEROKU_APP
}

function run_sanity_check() {

  # update .bashrc
  sed -i "s/^DIGESTER_PROMOTE_WORDLIST=.*/DIGESTER_PROMOTE_WORDLIST=\"all sanity\"/g" ~/.bashrc
  echo "You might have to reload your .bashrc"

  # check this script
  shellcheck -x "$MYSELF"
}


case $CMD in
  all)    promote_all ;;
  sanity) run_sanity_check ;;
  *)
    echo "unknown command.."
    exit 1
    ;;
esac
		
