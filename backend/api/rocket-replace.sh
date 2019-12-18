#!/bin/bash

# This script can be used to start a rocket application with
# environment variables setting values in Rocket.toml, because
# TOML doesn't support environment variables and Rocket only
# allows overriding variables by fully-qualifying them, which
# can get hairy.
#
# This script assumes that there is a Rocket.template.toml with
# some values set to ROCKET_REPLACE_XXX. Eg.
#
# Rocket.toml
#   [global.databases]
#   my-db-db = "ROCKET_REPLACE_DB_PW"
#
# This script will copy the Rocket.template.toml to Rocket.toml,
# replace all occurences of ROCKET_REPLACE with the value of the
# environment variable with the same name and eventually start
# the application.
#
#
# Usage:
#  ./rocket-replace.sh target/debug/myapp
#
#
# Preconditions:
#  - Rocket.toml must not exist in current directory
#  - The script must be called with the executable passed
#  - The script must replace all variables
#
# If any of these conditions is not met, the script exits with code 1.


set -e

TEMPLATE_ROCKET_FILE="Rocket.template.toml"
TARGET_ROCKET_FILE="Rocket.toml"

ROCKET_EXECUTABLE=$1

if [ ! -x "$ROCKET_EXECUTABLE" ]
then
    printf 'Error: No executable passed\n' >&2
    exit 1
fi


if [ -e ${TARGET_ROCKET_FILE} ]
then
    printf 'Error: Rocket.toml already exists\n' >&2
    exit 1
fi

eval 'vars=(${!'"ROCKET_REPLACE"'@})';


cp ${TEMPLATE_ROCKET_FILE} ${TARGET_ROCKET_FILE}

for var in "${vars[@]}"
do
    # rhs in sed must be escaped: https://unix.stackexchange.com/a/129063
    val=$(printf '%s\n' "${!var}" | sed 's:[\/&]:\\&:g;$!s/$/\\/');
    sed -i "s/$var/$val/" ${TARGET_ROCKET_FILE}
done

if grep --quiet ROCKET_REPLACE ${TARGET_ROCKET_FILE}
then
    printf 'Error: Not all variables replaced:\n' >&2
    grep ROCKET_REPLACE ${TARGET_ROCKET_FILE}
    exit 1
fi

# required for heroku, because they will pass $PORT
# where we need to bind to
export ROCKET_PORT=${PORT:-8000}

exec ${ROCKET_EXECUTABLE}
