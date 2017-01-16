defmodule TanukiIncoming.Mixfile do
  use Mix.Project

  def project do
    [app: :tanuki_incoming,
     version: "0.1.0",
     build_path: "../../_build",
     config_path: "../../config/config.exs",
     deps_path: "../../deps",
     lockfile: "../../mix.lock",
     elixir: "~> 1.4",
     build_embedded: Mix.env == :prod,
     start_permanent: Mix.env == :prod,
     deps: deps()]
  end

  def application do
    [applications: [
      :kernel,
      :stdlib,
      :mnesia,
      :mimetypes,
      :jsx,
      :couchbeam],
     extra_applications: [:logger],
     mod: {TanukiIncoming.Application, []},
     description: 'Digital assets import application.']
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.4.2"},
     {:emagick_rs, github: "nlfiedler/emagick.rs", tag: "0.4.4"},
     {:epwd_rs, github: "nlfiedler/epwd.rs", tag: "0.1.7"},
     {:mimetypes, github: "spawngrid/mimetypes", ref: "47d37a9"},
     {:tanuki_backend, in_umbrella: true}]
  end
end
