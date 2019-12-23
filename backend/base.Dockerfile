FROM rustlang/rust:nightly as build

RUN mkdir /tmp/digester-build
WORKDIR /tmp/digester-build

COPY ./ /tmp/digester-build
RUN cargo build --release