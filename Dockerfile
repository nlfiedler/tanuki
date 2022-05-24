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
RUN fvm install 2.10.5
WORKDIR /flutter
COPY fonts fonts/
COPY lib lib/
COPY pubspec.yaml .
COPY public public/
COPY web web/
RUN fvm use 2.10.5
RUN fvm flutter config --enable-web
RUN fvm flutter pub get
ENV BASE_URL ${BASE_URL}
RUN fvm flutter pub run environment_config:generate
# for now, no null safety until all dependencies build successfully with it
RUN fvm flutter build web --no-sound-null-safety

#
# build the final image
#
FROM debian:latest
WORKDIR /tanuki
COPY --from=builder /build/target/release/tanuki .
COPY --from=healthy /health/target/release/healthcheck .
COPY --from=flutter /flutter/build/web web/
VOLUME /assets
VOLUME /database
ENV DB_PATH "/database"
ENV UPLOAD_PATH "/assets/uploads"
ENV ASSETS_PATH "/assets/blobstore"
ENV HOST "0.0.0.0"
ENV PORT 3000
ENV RUST_LOG info
EXPOSE ${PORT}
HEALTHCHECK CMD ./healthcheck
ENTRYPOINT ["./tanuki"]
