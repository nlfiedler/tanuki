use Mix.Config

# General application configuration
config :tanuki_incoming,
  couchdb_url: 'http://localhost:5984',
  couchdb_opts: [],
  database: 'tanuki',
  incoming_dir: '/you/must/set/this'

# Import environment specific config, if it exists, to override the
# configuration defined above.
import_config "#{Mix.env}.exs"
