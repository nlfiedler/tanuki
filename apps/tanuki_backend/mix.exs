defmodule TanukiBackend.Mixfile do
  use Mix.Project

  def project do
    [app: :tanuki_backend,
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
    [extra_applications: [:mnesia, :logger],
     mod: {TanukiBackend.Application, []},
     description: 'Data access and caching layer.']
  end

  defp deps do
    [{:couchbeam_amuino, "~> 1.4.3-amuino.8"},
     {:emagick_rs, github: "nlfiedler/emagick.rs", tag: "0.5.0"},
     {:temp, "~> 0.4.3"}]
  end
end
