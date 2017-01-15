#
# Configuration overrides for testing
#
use Mix.Config

# Note that the :assets_dir and :incoming_dir are computed by the test
# suite since they are always local to the test run directory, so any
# existing setting is ignored at test time.
#
# As for :database, we want to use a database that can be clobbered without
# mercy, since that is exactly what the test suite will do.
config :tanuki_incoming,
  database: 'tanuki_test'
