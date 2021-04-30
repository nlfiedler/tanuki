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
# For consistency, use the Dart image as a base, add a version of Flutter that
# is known to work via the fvm tool, and then enable the web platform as a build
# target.
#
FROM google/dart AS flutter
ARG BASE_URL=http://localhost:3000
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get -q update && \
    apt-get -q -y install unzip
RUN pub global activate fvm
RUN fvm install stable
WORKDIR /flutter
COPY fonts fonts/
COPY lib lib/
COPY pubspec.yaml .
COPY web web/
RUN fvm use stable
RUN fvm flutter config --enable-web
RUN fvm flutter pub get
ENV BASE_URL ${BASE_URL}
RUN fvm flutter pub run environment_config:generate
RUN fvm flutter build web

#
# build the final image
#
FROM debian:latest
WORKDIR /tanuki
COPY --from=builder /build/target/release/tanuki .
COPY --from=healthy /health/target/release/healthcheck .
COPY --from=flutter /flutter/build/web web/
VOLUME /blobstore
VOLUME /database
VOLUME /uploads
ENV DB_PATH "/database"
ENV UPLOAD_PATH "/uploads"
ENV ASSETS_PATH "/blobstore"
ENV HOST "0.0.0.0"
ENV PORT 3000
EXPOSE ${PORT}
HEALTHCHECK CMD ./healthcheck
ENV RUST_LOG info
ENTRYPOINT ["./tanuki"]
