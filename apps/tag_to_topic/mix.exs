defmodule TagToTopic.Mixfile do
  use Mix.Project

  def project do
    [app: :tag_to_topic,
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
     mod: {:tag_to_topic, []},
     description: 'Remove a tag and set as topic in all matching documents.']
  end

  # $ cd apps/tag_to_topic
  # $ mix escript.build
  # $ ./tag_to_topic <config> <tag>
  defp escript do
    [main_module: :tag_to_topic,
     language: :erlang,
     app: nil]
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.3.1"}]
  end
end
