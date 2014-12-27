# Development Setup

## Prerequisites

While you could certainly perform the entire development cycle within your favorite environment, it makes some sense to use the Linux VM, to ensure everything works for the target system.

* [VirtualBox](https://www.virtualbox.org)
* [Vagrant](http://www.vagrantup.com)
* [Fabric](http://www.fabfile.org)

## Setup

```
$ cd vagrant
$ vagrant up
$ ./runfab build_all
```

## Development Cycle

As mentioned above, it is probably best to use the Linux VM for development, to ensure everything works for the target system. You could, of course, use any system on which Erlang and CouchDB are supported.

* Perform the setup for the VM as described above.
* Edit the code using your favorite editor on the host system
* From the Linux VM, run the following commands:
```
$ cd /vagrant_data
$ rebar compile
$ ./_rel/tanuki/bin/tanuki
```
* Visit [http://localhost:8000](http://localhost:8000) to see the web interface
    * Thanks to port-forwarding, you can use your favorite browser on the host system

## Prototype Incoming Processor

### Python 3.x

* Install Python 3.x.
* Install Hy(lang) http://hylang.org
* Install https://github.com/djc/couchdb-python
* Install https://github.com/ianare/exif-py

#### Detailed Steps

```
$ pyvenv-3.4 ~/Python/tanuki
$ . ~/Python/tanuki/bin/activate
$ pip install hy
$ git clone git@github.com:djc/couchdb-python.git
$ cd couchdb-python
$ python setup.py build
$ python setup.py install
$ git clone git@github.com:ianare/exif-py.git
$ cd exif-py
$ python setup.py build
$ python setup.py install
```

## Backend

### Requirements

* ImageMagick
    * Homebrew: `brew install imagemagick`
