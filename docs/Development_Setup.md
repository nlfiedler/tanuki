# Development Setup

## Prerequisites

* Erlang/OTP R17
    - Homebrew: `brew install erlang`
    - FreeBSD: `pkg install erlang`
    - Ubuntu Linux: build from source as shown in the `Dockerfile`
* CouchDB 1.6.1 or higher
    - Homebrew: `brew install couchdb`
    - FreeBSD: `pkg install couchdb`
    - Ubuntu Linux: build from source as shown in the `Dockerfile`
* ImageMagick
    - Homebrew: `brew install imagemagick`
    - FreeBSD: `pkg install ImageMagick-nox11`
    - Ubuntu Linux: `apt-get install imagemagick`

## Docker

If setting up a development environment for tanuki seems like too much work, there is a `Dockerfile` in the `docker` directory, which will build an Ubuntu Linux container to run tanuki. For this you will need [Docker](https://www.docker.com) installed, both to build and run the container. See the instructions at the top of the `Dockerfile` for some guidance on how to use it. If you are using Mac OS X, check out [boot2docker](http://boot2docker.io), which works very well.
