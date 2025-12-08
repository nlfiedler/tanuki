#
# Build the application container in stages.
#
# c.f. https://bun.com/docs/guides/ecosystem/docker
#
# use 1.3.3 for now, 1.3.4 resulted in "error: module not found"
FROM oven/bun:1.3.3 AS base
WORKDIR /home/bun/app

FROM base AS install
# prepare development dependencies in a temporary directory
RUN mkdir -p /temp/dev
COPY package.json bun.lock /temp/dev/
RUN cd /temp/dev && bun install --frozen-lockfile
# install production dependencies in another directory for a later stage
RUN mkdir -p /temp/prod
COPY package.json bun.lock /temp/prod/
RUN cd /temp/prod && bun install --frozen-lockfile --production

# build the front-end using the development dependencies
FROM base AS prerelease
COPY --from=install /temp/dev/node_modules node_modules
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
    apt-get -q -y install curl openssl ca-certificates
# copy production dependencies, server code, and compiled front-end into the final image
COPY --from=install /temp/prod/node_modules node_modules
COPY --from=prerelease /home/bun/app/dist dist
COPY --from=prerelease /home/bun/app/generated generated
COPY --from=prerelease /home/bun/app/server server
COPY --from=prerelease /home/bun/app/package.json .

# run the app
USER bun
VOLUME /assets
ENV PORT=3000
EXPOSE ${PORT}
HEALTHCHECK CMD curl --fail --silent --head http://localhost:${PORT}/liveness || exit 1
ENTRYPOINT [ "bun", "server/main.ts" ]
