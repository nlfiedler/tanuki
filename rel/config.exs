use Mix.Releases.Config,
  # This sets the default release built by `mix release`
  default_release: :default,
  # This sets the default environment used by `mix release`
  default_environment: :dev

# For a full list of config options for both releases and environments,
# visit https://hexdocs.pm/distillery/configuration.html

# You may define one or more environments in this file, an environment's
# settings will override those of a release when building in that
# environment, this combination of release and environment configuration is
# called a profile

environment :dev do
  set dev_mode: true
  set include_erts: false
  set cookie: :"monster"
end

environment :prod do
  set include_erts: false
  set include_src: false
  set cookie: :"pikachu"
  # This would be nice, but probably need a fallback when COOKIE is not defined.
  # set cookie: :"#{:crypto.hash(:sha256, System.get_env("COOKIE"))}"
end

# You may define one or more releases in this file. If you have not set a
# default release, or selected one when running `mix release`, the first
# release in the file will be used by default

release :tanuki do
  set version: "0.1.0"
  set applications: [
    tanuki_backend: :permanent,
    tanuki_incoming: :permanent,
    emagick_rs: :load,
    epwd_rs: :load
  ]
  set vm_args: "config/vm.args"
  # Need to figure this one out; may need to set overlay_vars as well.
  # set overlays: [
  #   copy: {"{{root_dir}}/apps/tanuki_backend/priv/views", "{{output_dir}}/priv/views"},
  #   copy: {"_build/prod/lib/tanuki_backend/Version", "{{output_dir}}/Version"}
  # ]
end
