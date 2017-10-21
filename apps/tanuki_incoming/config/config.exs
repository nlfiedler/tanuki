use Mix.Config

# General application configuration
config :tanuki_incoming,
  couchdb_url: 'http://localhost:5984',
  couchdb_opts: [],
  database: 'tanuki',
  incoming_dir: '/you/must/set/this',
  frequency: 60  # how often to check for new assets, in minutes

# Configure mimetypes to know about the relatively new High Efficiency Image
# File Format that iOS now produces by default. See mimetypes issue #27 for
# details (https://github.com/erlangpack/mimetypes/issues/27).
config :mimetypes,
  load: [
    {:default, [
      {"heic", "image/heic"},
      {"heif", "image/heif"},
    ]}
  ]

# Import environment specific config, if it exists, to override the
# configuration defined above.
import_config "#{Mix.env}.exs"
