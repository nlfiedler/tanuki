#
# Build the application container in stages.
#
# Previously had used the oven/bun image but that started failing with a "module
# not found" error with the 1.3.4 release; could not determine the cause or fix.
#
FROM debian:latest AS base
ENV DEBIAN_FRONTEND="noninteractive"
# The bun install script itself needs curl to fetch the zip file from
# github.com; while manually installing Bun is not too difficult, the install
# script is sure to work regardless of what changes are made in the future. Set
# BUN_INSTALL so the subsequent stages can find the bun executable.
RUN apt-get -q update && \
    apt-get -q -y install curl unzip
ENV BUN_INSTALL="/usr/local"
RUN curl -fsSL https://bun.com/install | bash
WORKDIR /app

FROM base AS install
# prepare development dependencies in a temporary directory
RUN mkdir -p /build/dev
COPY package.json bun.lock /build/dev/
RUN cd /build/dev && bun install --frozen-lockfile
# install production dependencies in another directory for a later stage
RUN mkdir -p /build/prod
COPY package.json bun.lock /build/prod/
RUN cd /build/prod && bun install --frozen-lockfile --production

# build the front-end using the development dependencies
FROM base AS prerelease
COPY --from=install /build/dev/node_modules node_modules
COPY client client
COPY public public
COPY server server
COPY codegen.ts codegen.ts
COPY index.html index.html
COPY package.json package.json
COPY vite.config.ts vite.config.ts
ENV NODE_ENV=production
RUN bun run codegen
RUN bunx --bun vite build

FROM base AS release
# ensure SSL and CA certificates are available for HTTPS clients
ENV DEBIAN_FRONTEND="noninteractive"
RUN apt-get -q update && \
    apt-get -q -y install openssl ca-certificates
# copy production dependencies, server code, and compiled front-end into the final image
COPY --from=install /build/prod/node_modules node_modules
COPY --from=prerelease /app/dist dist
COPY --from=prerelease /app/generated generated
COPY --from=prerelease /app/server server
COPY --from=prerelease /app/package.json .
COPY containers/healthcheck.ts healthcheck.ts

# run the app
VOLUME /assets
ENV PORT=3000
ENV HEALTHCHECK_PATH="/liveness"
EXPOSE ${PORT}
HEALTHCHECK CMD ["bun", "run", "healthcheck.ts"]
ENTRYPOINT [ "bun", "run", "server/main.ts" ]
