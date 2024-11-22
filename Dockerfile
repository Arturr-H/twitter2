# This file is used to build the rust backend image
# and does not include postgres because it will be
# provided in the docker-compose file as a service
FROM docker.io/rust:latest as builder
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
# to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/twitter2*

# Copy the rest of the source code
COPY . .

# Needed to cause SQLX to disable their compile time
# macro checks which require a database connection
ARG SQLX_OFFLINE=true
COPY migrations ./migrations
COPY .sqlx ./.sqlx
RUN cargo build --release

FROM ubuntu:24.10
# COPY ./assets/images/profile /usr/src/app/assets/images/profile
COPY --from=builder /usr/src/app/target/release/twitter2 /usr/local/bin/twitter2
COPY .env .
COPY migrations ./migrations
COPY .sqlx ./.sqlx
EXPOSE 8081
CMD ["twitter2"]
