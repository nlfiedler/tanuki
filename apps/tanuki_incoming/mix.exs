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
    [applications: [
      :kernel,
      :stdlib,
      :logger,
      :mnesia,
      :jsx,
      :couchbeam],
     included_applications: [
      :mimerl
     ],
     mod: {TanukiIncoming.Application, []},
     description: 'Digital assets import application.']
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.4.2"},
     {:exif, github: "nlfiedler/erlang-exif", tag: "2.0.3"},
     {:epwd_rs, github: "nlfiedler/epwd.rs", tag: "0.1.8"},
     # hackney uses an older mimerl, but should be okay to use a newer release
     {:mimerl, github: "benoitc/mimerl", tag: "1.1.1", override: true},
     {:poison, "~> 3.1"},
     {:tanuki_backend, in_umbrella: true}]
  end
end
