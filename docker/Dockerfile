#
# Dockerfile for testing tanuki on Ubuntu Linux.
# Steps below assume the use of boot2docker on Mac OS X.
#
# ## Build and Run
#
# $ docker build -t tanuki docker
# $ docker run --privileged -d -v $HOME:/mnt/shared -p 2022:22 -p 8984:5984 -p 8000:8000 tanuki
#
# ## Connect via SSH
#
# $ ssh -p 2022 -i docker/charlie_insecure charlie@$(boot2docker ip)
#
# ## Connect to CouchDB
#
# $ curl http://$(boot2docker ip):8984/
# {"couchdb":"Welcome","uuid":"efb216878f19d21b538c0df7766b5435","version":"1.6.1"...}}
#
FROM ubuntu:trusty
MAINTAINER nathanfiedler@fastmail.fm

# set root password to something known
RUN echo 'root:p8ssword' | chpasswd

# install sshd, supervisord, and build essentials
ENV DEBIAN_FRONTEND noninteractive
# need sources.list for apt-get -y build-dep ...
RUN echo "deb-src http://archive.ubuntu.com/ubuntu trusty main universe" >> /etc/apt/sources.list
RUN apt-get -q update
RUN apt-get -q -y install build-essential openssh-server supervisor
RUN mkdir -p /var/run/sshd
RUN mkdir -p /var/log/supervisor
ADD supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# boot2docker has a very minimal base system, ensure these packages are present
RUN apt-get -q -y install curl emacs24-nox less net-tools sudo vim xz-utils

# generate the default locale to avoid errors
RUN locale-gen en_US.UTF-8

# build the latest Erlang/OTP from source
RUN apt-get install -q -y libncurses5-dev libssl-dev
ADD http://www.erlang.org/download/otp_src_17.5.tar.gz /tmp/
RUN tar -C /tmp -z -x -f /tmp/otp_src_17.5.tar.gz
RUN cd /tmp/otp_src_17.5 && ./configure && make && make install
RUN rm -rf /tmp/otp_src_17.3*

# install CouchDB and its dependencies
RUN apt-get install -q -y libcurl4-openssl-dev libicu-dev libmozjs185-dev
ADD http://mirrors.ibiblio.org/apache/couchdb/source/1.6.1/apache-couchdb-1.6.1.tar.gz /tmp/
RUN tar -C /tmp -z -x -f /tmp/apache-couchdb-1.6.1.tar.gz
RUN cd /tmp/apache-couchdb-1.6.1 && ./configure && make && make install
RUN rm -rf apache-couchdb-1.6.1*

# configure CouchDB
RUN adduser --gecos "CouchDB Administrator" --group --home /usr/local/var/lib/couchdb --no-create-home --shell /bin/bash --system couchdb
RUN ln -s /usr/local/etc/logrotate.d/couchdb /etc/logrotate.d/couchdb
RUN ln -s /usr/local/etc/init.d/couchdb  /etc/init.d
RUN chown -R couchdb:couchdb /usr/local/etc/couchdb
RUN chown -R couchdb:couchdb /usr/local/var/lib/couchdb
RUN chown -R couchdb:couchdb /usr/local/var/log/couchdb
RUN chown -R couchdb:couchdb /usr/local/var/run/couchdb
RUN chmod 0770 /usr/local/etc/couchdb
RUN chmod 0770 /usr/local/var/lib/couchdb
RUN chmod 0770 /usr/local/var/log/couchdb
RUN chmod 0770 /usr/local/var/run/couchdb
ADD couchdb.ini /usr/local/etc/couchdb/local.d/couchdb.ini
RUN chown couchdb:couchdb /usr/local/etc/couchdb/local.d/couchdb.ini
RUN mkdir -p /usr/local/var/couchdb
RUN chown couchdb:couchdb /usr/local/var/couchdb
RUN update-rc.d couchdb defaults

# # install git, rebar, and relx for developing tanuki
RUN apt-get install -q -y git

RUN git clone -q git://github.com/rebar/rebar.git
RUN cd rebar && ./bootstrap && cp rebar /usr/local/bin
RUN rm -rf rebar

RUN git clone -q https://github.com/erlware/relx.git
RUN cd relx && ./rebar3 escriptize && cp _build/default/bin/relx /usr/local/bin
RUN rm -rf relx

# install ImageMagick for thumbnail generation
RUN apt-get install -q -y imagemagick

# add charlie user with password-less sudo
RUN adduser --disabled-password --gecos '' charlie
RUN adduser charlie sudo
RUN echo 'charlie:p8ssword' | chpasswd
ADD charlie_sudoers /etc/sudoers.d/charlie
RUN chmod 0440 /etc/sudoers.d/charlie
# Weird sequence to get around https://github.com/docker/docker/issues/1295
# (create .ssh directory first, set ownership, then populate)
RUN mkdir /home/charlie/.ssh
RUN chown -R charlie:charlie /home/charlie/.ssh
ADD charlie_insecure.pub /home/charlie/.ssh/authorized_keys
RUN chmod 0700 /home/charlie/.ssh
RUN chmod 0600 /home/charlie/.ssh/authorized_keys
RUN chown -R charlie:charlie /home/charlie

# open the sshd, CouchDB, and Nitrogen ports
EXPOSE 22 5984 8000

# start supervisord in the foreground
CMD ["/usr/bin/supervisord"]
