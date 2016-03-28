# -*- coding: utf-8 -*-
# -------------------------------------------------------------------
#
# Copyright (c) 2016 Nathan Fiedler
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
"""Fabric file for installing requirements on FreeBSD."""

import ConfigParser
import os
import time

from fabric.api import env, put, run, sudo, task

env.shell = "/bin/sh -c"
env.hosts = ["default"]
env.use_ssh_config = True
if os.path.exists("user_ssh_config"):
    env.ssh_config_path = "user_ssh_config"
else:
    env.ssh_config_path = "ssh_config"


@task
def all():
    """Install everything needed to build magick-rust."""
    sudo("pkg install -q -y git")
    sudo("pkg install -q -y rust")
    sudo("pkg install -q -y cargo")
    sudo("pkg install -q -y ImageMagick-nox11")
    sudo("pkg install -q -y erlang")
    sudo('pkg install -q -y rebar')
    sudo('pkg install -q -y relx')
    install_clang()
    install_couchdb()


@task
def install_clang():
    """Install Clang and ensure it can be found by rustc."""
    sudo("pkg install -q -y clang-devel")
    # set LIBCLANG_PATH so rustc can find libclang.so in its hidden place
    run("echo 'setenv LIBCLANG_PATH /usr/local/llvm-devel/lib' >> .cshrc")


@task
def install_couchdb():
    """Install Erlang/OTP and Apache CouchDB."""
    sudo("pkg install -q -y couchdb")
    couchdb_ini = 'couchdb.ini'
    if os.path.exists('user_couchdb.ini'):
        couchdb_ini = 'user_couchdb.ini'
    put(couchdb_ini, 'couchdb.ini')
    sudo('mv couchdb.ini /usr/local/etc/couchdb/local.d/couchdb.ini')
    sudo('chown couchdb:wheel /usr/local/etc/couchdb/local.d/couchdb.ini')
    # create the database directory based on our custom configuration
    couchdb_conf = ConfigParser.ConfigParser()
    couchdb_conf.read(couchdb_ini)
    if couchdb_conf.has_option('couchdb', 'database_dir'):
        database_dir = couchdb_conf.get('couchdb', 'database_dir')
        sudo('mkdir -p {}'.format(database_dir), quiet=True)
        sudo('chown couchdb:couchdb {}'.format(database_dir))
    # enable and start the service
    sudo("echo couchdb_enable='YES' >> /etc/rc.conf")
    sudo("echo couchdb_enablelogs='YES' >> /etc/rc.conf")
    sudo("echo couchdb_user='couchdb' >> /etc/rc.conf")
    sudo('service couchdb start')
    # give CouchDB a moment to get started
    time.sleep(1)
    run('curl http://127.0.0.1:5984/')
