FROM node:lts

ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update
RUN apt-get -q -y install apt-utils

WORKDIR /kuukan

#
# using BuckleScript requires gcc/make to compile OCaml
#
RUN apt-get -q -y install build-essential
RUN npm install -q -g gulp-cli

#
# install build-time dependencies
#
RUN apt-get install -q -y python

#
# Copy the application code and build.
#
COPY bsconfig.json .
COPY config config
COPY graphql_schema.json .
COPY gulpfile.js .
COPY package.json .
COPY package-lock.json .
COPY public public
COPY src src
COPY views views
RUN npm ci
RUN gulp build

# node container has a "node" user that owns everything?
USER node

VOLUME /tanuki

EXPOSE ${PORT}

COPY docker/healthcheck.js .
HEALTHCHECK CMD node ./healthcheck.js ${PORT}

ENTRYPOINT [ "node",  "./src/server.js" ]
