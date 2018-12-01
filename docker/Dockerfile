FROM node:10

ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update
RUN apt-get -q -y install apt-utils

WORKDIR /src

#
# installing BuckleScript requires gcc/make to compile OCaml;
# the --unsafe-perm is to get around permissions errors that
# are quite common with the bs-platform installation
#
RUN apt-get -q -y install build-essential
RUN npm install -q -g bs-platform --unsafe-perm
RUN npm install -q -g gulp-cli

#
# install build-time dependencies
#
RUN apt-get install -q -y python

#
# Copy the application code and build.
#
COPY app.js .
COPY bin bin
COPY bsconfig.json .
COPY config config
COPY graphql_schema.json .
COPY gulpfile.js .
COPY lib lib
COPY package.json .
COPY public public
COPY routes routes
COPY src src
COPY views views
RUN npm install -q
RUN gulp compile

# node container has a "node" user that owns everything?
USER node

VOLUME /tanuki

EXPOSE ${PORT}

COPY docker/healthcheck.js .
HEALTHCHECK CMD node ./healthcheck.js ${PORT}

ENTRYPOINT [ "node",  "bin/www" ]
