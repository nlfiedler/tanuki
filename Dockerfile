#
# build the application binaries
#
FROM rust:latest AS builder
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get -q update && \
    apt-get -q -y install clang
WORKDIR /build
COPY Cargo.toml .
COPY src src/
RUN cargo build --release

#
# build the healthcheck binary
#
FROM rust:latest AS healthy
WORKDIR /health
COPY healthcheck/Cargo.toml .
COPY healthcheck/src src/
RUN cargo build --release

#
# build the flutter app
#
# Use their "beta" tag, but also set the channel to beta anyway, and make sure
# it is completely up-to-date, and then enable the web channel as well.
#
FROM cirrusci/flutter:beta AS flutter
ARG BASE_URL=http://localhost:8080
RUN flutter channel beta
RUN flutter upgrade
RUN flutter config --enable-web
WORKDIR /flutter
COPY fonts fonts/
COPY lib lib/
COPY pubspec.yaml .
COPY web web/
RUN flutter pub get
ENV BASE_URL ${BASE_URL}
RUN flutter pub run environment_config:generate
RUN flutter build web

#
# build the final image
#
FROM debian:latest
RUN adduser --disabled-password --gecos '' tanuki
USER tanuki
WORKDIR /tanuki
COPY --from=builder /build/target/release/tanuki .
COPY --from=healthy /health/target/release/healthcheck .
COPY --from=flutter /flutter/build/web web/
VOLUME /blobstore
VOLUME /database
VOLUME /uploads
ENV HOST "0.0.0.0"
ENV PORT 3000
EXPOSE ${PORT}
HEALTHCHECK CMD ./healthcheck
ENV RUST_LOG info
ENTRYPOINT ["./tanuki"]
