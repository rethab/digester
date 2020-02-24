FROM library/postgres:12.1
COPY init.sql /docker-entrypoint-initdb.d/
