defmodule CorrectOrientation.Mixfile do
  use Mix.Project

  def project do
    [app: :correct_orientation,
     version: "1.0.0",
     build_path: "../../_build",
     config_path: "../../config/config.exs",
     deps_path: "../../deps",
     lockfile: "../../mix.lock",
     compilers: [:erlang, :app],
     erlc_options: [:debug_info, :fail_on_warning],
     escript: escript,
     deps: deps]
  end

  def application do
    [applications: [
      :kernel,
      :stdlib,
      :couchbeam,
      :hackney,
      :idna,
      :jsx,
      :ssl_verify_fun],
     mod: {:correct_orientation, []},
     description: 'Correct orientation of all image assets.']
  end

  # $ cd apps/correct_orientation
  # $ mix escript.build
  # $ ./correct_orientation <config>
  defp escript do
    [main_module: :correct_orientation,
     language: :erlang,
     app: nil]
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.3.1"},
     {:emagick_rs, github: "nlfiedler/emagick.rs", tag: "0.4.4"}]
  end
end
