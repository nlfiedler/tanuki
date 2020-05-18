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
# build the frontend
#
FROM node:lts AS webster
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get -q update && \
    apt-get -q -y install apt-utils build-essential python
RUN npm install -q -g gulp-cli
WORKDIR /webster
COPY bsconfig.json .
COPY graphql_schema.json .
COPY gulpfile.js .
COPY package.json .
COPY package-lock.json .
COPY public public/
COPY src/*.re src/
RUN npm ci
ENV BUILD_ENV production
RUN gulp build

#
# build the final image
#
FROM debian:latest
RUN adduser --disabled-password --gecos '' tanuki
USER tanuki
WORKDIR /tanuki
COPY --from=builder /build/target/release/tanuki .
COPY --from=healthy /health/target/release/healthcheck .
COPY --from=webster /webster/public public/
VOLUME /blobstore
VOLUME /database
VOLUME /uploads
ENV HOST "0.0.0.0"
ENV PORT 3000
EXPOSE ${PORT}
HEALTHCHECK CMD ./healthcheck
ENV RUST_LOG info
ENTRYPOINT ["./tanuki"]
