# compute a lock-like file for our project
FROM lukemathwalker/cargo-chef as planner
WORKDIR app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

# build only the  project dependencies
FROM lukemathwalker/cargo-chef as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build our application, leveraging the cached deps!
FROM rust:1.49 AS builder
WORKDIR app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin newsletter

# runtime stage
FROM debian:buster-slim AS runtime
WORKDIR app
# install OpenSSL because it is dynamically linked by some of our dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# the binary is statically compiled
COPY --from=builder /app/target/release/newsletter newsletter
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./newsletter"]