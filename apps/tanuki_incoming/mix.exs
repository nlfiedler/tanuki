defmodule TanukiIncoming.Mixfile do
  use Mix.Project

  def project do
    [app: :tanuki_incoming,
     version: "0.1.0",
     build_path: "../../_build",
     config_path: "../../config/config.exs",
     deps_path: "../../deps",
     lockfile: "../../mix.lock",
     elixir: "~> 1.3",
     build_embedded: Mix.env == :prod,
     start_permanent: Mix.env == :prod,
     deps: deps()]
  end

  def application do
    [extra_applications: [:logger],
     mod: {TanukiIncoming.Application, []},
     description: 'Digital assets import application.']
  end

  defp deps do
    [{:couchbeam_amuino, "~> 1.4.3-amuino.8"},
     {:emagick_rs, github: "nlfiedler/emagick.rs", tag: "0.5.0"},
     {:epwd_rs, github: "nlfiedler/epwd.rs", tag: "0.1.9", runtime: false},
     {:mimetypes, "~> 1.1"},
     {:poison, "~> 3.1"},
     {:tanuki_backend, in_umbrella: true}]
  end
end
