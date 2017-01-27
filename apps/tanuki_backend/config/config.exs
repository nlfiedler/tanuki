# This file is responsible for configuring your application and its
# dependencies with the aid of the Mix.Config module.
#
# This configuration file is loaded before any dependency and is restricted
# to this project.
use Mix.Config

# General application configuration
config :tanuki_backend,
  assets_dir: '/usr/local/var/tanuki/assets',
  # generated thumbnails and previews will be in this directory
  thumbnails_dir: '/usr/local/var/tanuki/thumbnails',
  # number of thumbnail images to retain (in thumbnails_dir/240)
  thumbnails_limit: 512,
  # number of preview images to retain (in thumbnails_dir/640)
  previews_limit: 16,
  couchdb_url: 'http://localhost:5984',
  couchdb_opts: [],
  database: 'tanuki'

# Import environment specific config, if it exists, to override the
# configuration defined above.
import_config "#{Mix.env}.exs"
