defmodule TanukiIncoming.Mixfile do
  use Mix.Project

  def project do
    [app: :tanuki_incoming,
     version: "0.1.0",
     build_path: "../../_build",
     config_path: "../../config/config.exs",
     deps_path: "../../deps",
     lockfile: "../../mix.lock",
     compilers: [:erlang, :app],
     erlc_options: [:debug_info,
                    :fail_on_warning,
                    {:parse_transform, :lager_transform}],
     deps: deps()]
  end

  def application do
    [applications: [
      :kernel,
      :stdlib,
      :lager,
      :mnesia,
      :mimetypes,
      :jsx,
      :couchbeam],
     mod: {:tanuki_incoming_app, []},
     description: 'Digital assets import application.']
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.3.1"},
     {:emagick_rs, github: "nlfiedler/emagick.rs", tag: "0.4.4"},
     {:epwd_rs, github: "nlfiedler/epwd.rs", tag: "0.1.7"},
     {:lager, github: "basho/lager", tag: "3.2.1"},
     {:mimetypes, github: "spawngrid/mimetypes", ref: "47d37a9"},
     {:tanuki_backend, in_umbrella: true}]
  end
end
