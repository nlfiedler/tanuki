# This file is responsible for configuring your application and its
# dependencies with the aid of the Mix.Config module.
#
# This configuration file is loaded before any dependency and is restricted
# to this project.
use Mix.Config

# General application configuration
config :tanuki_backend,
  assets_dir: '/usr/local/var/tanuki/assets',
  couchdb_url: 'http://localhost:5984',
  couchdb_opts: [],
  database: 'tanuki'

# Import environment specific config, if it exists, to override the
# configuration defined above.
import_config "#{Mix.env}.exs"
