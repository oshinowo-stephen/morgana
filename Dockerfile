FROM rust:1-buster AS docker_builder

WORKDIR /usr/binder/build

COPY Cargo.toml ./
COPY binder-entities ./binder-entities
COPY binder-server ./binder-server
COPY binder-utils ./binder-utils
COPY binder-fm ./binder-fm

# Default arguments
ARG MAIN_CONTAINER_PATH="bin"

ENV MAIN_CONTAINER_PATH=$MAIN_CONTAINER_PATH

ENV DATABASE_URL=${DATABASE_URL}
ENV BINDER_ADDRESS=${BINDER_ADDRESS}
ENV MAIN_CONTAINER_LIMIT=${MAIN_CONTAINER_LIMIT}
ENV BINDER_STORAGE_TOKEN=${BINDER_STORAGE_TOKEN}
ENV RUST_ENV="production"

RUN mkdir -p ${MAIN_CONTAINER_PATH}
RUN cargo build --release

CMD target/release/binder-server
