defmodule TanukiBackend.Mixfile do
  use Mix.Project

  def project do
    [app: :tanuki_backend,
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
      :jsx,
      :couchbeam],
     extra_applications: [:logger],
     mod: {TanukiBackend.Application, []},
     description: 'Data access and caching layer.']
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.4.2"},
     {:emagick_rs, github: "nlfiedler/emagick.rs", tag: "0.4.4"},
     {:temp, "~> 0.4.2", only: [:test]}]
  end
end
