use Mix.Config

config :tanuki_backend,
  couchdb_url: 'http://localhost:5984',
  couchdb_opts: [],
  database: 'tanuki_test'

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :tanuki_web, TanukiWeb.Web.Endpoint,
  http: [port: 4001],
  server: false

# Print only warnings and errors during test
config :logger, level: :warn
