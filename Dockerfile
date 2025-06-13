#
# build the application binaries
#
FROM rust:latest AS builder
ENV DEBIAN_FRONTEND="noninteractive"
RUN apt-get -q update && \
    apt-get -q -y install clang
# use --locked to prevent errors when incompatible crates get pulled in
RUN cargo install --locked cargo-leptos
RUN rustup target add wasm32-unknown-unknown
WORKDIR /build
COPY assets assets/
COPY Cargo.toml .
COPY src src/
COPY style style/
RUN cargo leptos build --release

#
# build the healthcheck binary
#
FROM rust:latest AS healthy
WORKDIR /build
COPY healthcheck/Cargo.toml .
COPY healthcheck/src src/
RUN cargo build --release

#
# build the final image
#
# ensure SSL and CA certificates are available for HTTPS client
#
FROM debian:latest
ARG SITE_ADDR="0.0.0.0:3000"
ENV DEBIAN_FRONTEND="noninteractive"
RUN apt-get -q update && \
    apt-get -q -y install openssl ca-certificates
WORKDIR /app
COPY --from=builder /build/target/release/tanuki .
COPY --from=builder /build/target/site site
COPY --from=builder /build/Cargo.toml .
COPY --from=healthy /build/target/release/healthcheck .
VOLUME /assets
VOLUME /database
ENV DATABASE_PATH="/database"
ENV DATABASE_TYPE="rocksdb"
ENV UPLOAD_PATH="/assets/uploads"
ENV ASSETS_PATH="/assets/blobstore"
ENV HEALTHCHECK_PATH="/liveness"
ENV LEPTOS_SITE_ADDR="${SITE_ADDR}"
ENV LEPTOS_SITE_ROOT="site"
ENV RUST_LOG="info"
EXPOSE ${PORT}
HEALTHCHECK CMD ./healthcheck
ENTRYPOINT ["./tanuki"]
