FROM node:12
#
# build the final image
#
# * using BuckleScript requires gcc/make to compile OCaml
# * some part of the build requires python
#
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get -q update && \
    apt-get -q -y install apt-utils build-essential python
RUN npm install -q -g gulp-cli

#
# Copy the application code and build.
#
WORKDIR /working
COPY bsconfig.json .
COPY config config/
COPY graphql_schema.json .
COPY gulpfile.js .
COPY package.json .
COPY package-lock.json .
COPY public public/
COPY src src/
COPY views views/
RUN npm ci
RUN gulp build

# node container has a "node" user that owns everything?
USER node

VOLUME /tanuki

EXPOSE ${PORT}

COPY healthcheck.js .
HEALTHCHECK CMD node ./healthcheck.js ${PORT}

ENTRYPOINT [ "node", "./src/server.js" ]
