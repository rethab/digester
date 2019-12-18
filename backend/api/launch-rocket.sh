#!/bin/bash

# This script can be used to start a rocket application with
# environment variables setting values in Rocket.toml, because
# TOML doesn't support environment variables and Rocket only
# allows overriding variables by fully-qualifying them, which
# can get hairy.
#
# This script assumes that there is a Rocket.toml with
# some values set to ROCKET_REPLACE_XXX. Eg.
#
# Rocket.toml
#   [global.databases]
#   my-db-db = "ROCKET_REPLACE_DB_PW"
#
# This script will replace all occurences of ROCKET_REPLACE in 
# Rocket.toml with the value of the environment variable with
# the same name and start the application.
#
#
# Usage (with replacements):
#  ./launch-rocket.sh --app target/debug/myapp
# (without replacements):
#  ./launch-rocket.sh --no-replace --app target/debug/myapp
#
#
# Preconditions:
#  - Rocket.toml must exist in current directory
#  - The script must be called with the executable passed
#  - The script must replace all variables
#
# If any of these conditions is not met, the script exits with code 1.


set -e

ROCKET_FILE="Rocket.toml"
REPLACE=true

while [[ "$#" -gt 0 ]]; do case $1 in
  --no-replace) REPLACE=false;;
  --app)        APP="$2"; shift;;
  *)            printf 'Error: Unknown parameter: %s\n' $1 >&2; exit 1;;
esac; shift; done

if [ ! -x "$APP" ]
then
    printf 'Error: No executable passed\n' >&2;
    exit 1;
fi

if [ ! -e ${ROCKET_FILE} ]
then
    printf 'Error: %s missing\n' ${ROCKET_FILE} >&2;
    exit 1;
fi

if [ "$REPLACE" = true ]
then
  printf 'Replace: yes\n';

  eval 'vars=(${!'"ROCKET_REPLACE"'@})';
  
  for var in "${vars[@]}"
  do
      # rhs in sed must be escaped: https://unix.stackexchange.com/a/129063
      val=$(printf '%s\n' "${!var}" | sed 's:[\/&]:\\&:g;$!s/$/\\/');
      sed -i "s/$var/$val/" ${ROCKET_FILE};
  done
  
  if grep --quiet ROCKET_REPLACE ${ROCKET_FILE}
  then
      printf 'Error: Not all variables replaced:\n' >&2;
      grep ROCKET_REPLACE ${ROCKET_FILE}
      exit 1;
  fi
else
  printf 'Replace: no\n';
fi

# required for heroku, because they will pass $PORT
# where we need to bind to
export ROCKET_PORT=${PORT:-8000}

exec ${APP}
