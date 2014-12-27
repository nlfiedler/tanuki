# -*- coding: utf-8 -*-
# -------------------------------------------------------------------
#
# Copyright (c) 2014 Nathan Fiedler
#
# This file is provided to you under the Apache License,
# Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain
# a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied. See the License for the
# specific language governing permissions and limitations
# under the License.
#
# -------------------------------------------------------------------
"""Fabric file for installing requirements on Ubuntu Linux."""

import ConfigParser
import time

from fabric.api import cd, path, put, run, shell_env, sudo, task

DIR_FOP = 'fop-1.1'
TAR_FOP = '{}-bin.tar.gz'.format(DIR_FOP)
URL_FOP = 'http://mirrors.sonic.net/apache/xmlgraphics/fop/binaries/{}'.format(TAR_FOP)
DIR_OTP = 'otp_src_17.3'
TAR_OTP = '{}.tar.gz'.format(DIR_OTP)
URL_OTP = 'http://www.erlang.org/download/{}'.format(TAR_OTP)
DIR_CDB = 'apache-couchdb-1.6.1'
TAR_CDB = '{}.tar.gz'.format(DIR_CDB)
URL_CDB = 'http://mirrors.sonic.net/apache/couchdb/source/1.6.1/{}'.format(TAR_CDB)
DIR_GIT = 'git-2.1.0'
TAR_GIT = '{}.tar.xz'.format(DIR_GIT)
URL_GIT = 'https://www.kernel.org/pub/software/scm/git/{}'.format(TAR_GIT)

# TODO: install ImageMagick


@task
def build_all():
    """Install everything needed to build tanuki."""
    install_erlang()
    install_couchdb()
    install_git()
    install_rebar()
    install_relx()


@task
def install_erlang():
    """Install Erlang/OTP from source tarball (installs tools & JDK)."""
    # Install the compilers, JDK, and XML tools
    pre_reqs = [
        'build-essential',
        'default-jdk',
        'libncurses5-dev',
        'libssl-dev',
        'libxml2-utils',
        'xsltproc',
    ]
    sudo('apt-get install -q -y {}'.format(' '.join(pre_reqs)))
    # Fetch and extract Apache FOP binary tarball
    run('wget -q {}'.format(URL_FOP))
    run('tar zxf {}'.format(TAR_FOP))
    # Prepare to build Erlang/OTP from source
    run('wget -q {}'.format(URL_OTP))
    run('tar zxf {}'.format(TAR_OTP))
    path_addend = '~/{}'.format(DIR_FOP)
    with cd(DIR_OTP), path(path_addend):
        run('./configure')
        run('make')
        sudo('make install')
        with shell_env(FOP_OPTS="-Xmx512m"):
            run('make docs')
            sudo('make install-docs')
    run('rm -rf {}* {}*'.format(DIR_FOP, DIR_OTP))


@task
def install_couchdb():
    """Install Apache CouchDB and dependencies (requires Erlang/OTP)."""
    # Install ICU and SpiderMonkey
    pre_reqs = [
        'libcurl4-openssl-dev',
        'libicu-dev',
        'libmozjs185-dev',
    ]
    sudo('apt-get install -q -y {}'.format(' '.join(pre_reqs)))
    run('wget -q {}'.format(URL_CDB))
    run('tar zxf {}'.format(TAR_CDB))
    with cd(DIR_CDB):
        run('./configure')
        run('make')
        sudo('make install')
    run('rm -rf {}*'.format(DIR_CDB))
    user_opts = [
        '--gecos "CouchDB Administrator"',
        '--group',
        '--home /usr/local/var/lib/couchdb',
        '--no-create-home',
        '--shell /bin/bash',
        '--system',
    ]
    sudo('adduser {0} couchdb'.format(' '.join(user_opts)))
    sudo('ln -s /usr/local/etc/logrotate.d/couchdb /etc/logrotate.d/couchdb')
    sudo('ln -s /usr/local/etc/init.d/couchdb  /etc/init.d')
    sudo('chown -R couchdb:couchdb /usr/local/etc/couchdb')
    sudo('chown -R couchdb:couchdb /usr/local/var/lib/couchdb')
    sudo('chown -R couchdb:couchdb /usr/local/var/log/couchdb')
    sudo('chown -R couchdb:couchdb /usr/local/var/run/couchdb')
    sudo('chmod 0770 /usr/local/etc/couchdb')
    sudo('chmod 0770 /usr/local/var/lib/couchdb')
    sudo('chmod 0770 /usr/local/var/log/couchdb')
    sudo('chmod 0770 /usr/local/var/run/couchdb')
    put('couchdb.ini', '/usr/local/etc/couchdb/local.d/couchdb.ini', use_sudo=True)
    sudo('chown couchdb:couchdb /usr/local/etc/couchdb/local.d/couchdb.ini')
    # create the database directory based on our custom configuration
    couch_conf = ConfigParser.ConfigParser()
    couch_conf.read('couchdb.ini')
    database_dir = couch_conf.get('couchdb', 'database_dir')
    sudo('mkdir -p {}'.format(database_dir), quiet=True)
    sudo('chown couchdb:couchdb {}'.format(database_dir))
    sudo('update-rc.d couchdb defaults')
    sudo('service couchdb start')
    # give CouchDB a moment to get started
    time.sleep(1)
    run('curl http://127.0.0.1:5984/')


@task
def install_git():
    """Build and install Git."""
    sudo('apt-get install -q -y git')


@task
def install_rebar():
    """Build and install the rebar build tool."""
    run('git clone -q git://github.com/rebar/rebar.git')
    with cd('rebar'):
        run('./bootstrap')
        sudo('cp rebar /usr/local/bin')
    run('rm -rf rebar')


@task
def install_relx():
    """Build and install the relx release tool."""
    run('git clone -q https://github.com/erlware/relx.git')
    with cd('relx'):
        run('make')
        sudo('cp relx /usr/local/bin')
    run('rm -rf relx')
