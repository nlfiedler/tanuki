defmodule MergeRecords.Mixfile do
  use Mix.Project

  def project do
    [app: :merge_records,
     version: "1.0.0",
     build_path: "../../_build",
     config_path: "../../config/config.exs",
     deps_path: "../../deps",
     lockfile: "../../mix.lock",
     compilers: [:erlang, :app],
     erlc_options: [:debug_info, :fail_on_warning],
     escript: escript(),
     deps: deps()]
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
     mod: {:merge_records_app, []},
     description: 'Merge records that refer to the same asset.']
  end

  # $ cd apps/merge_records
  # $ mix escript.build
  # $ ./merge_records <config>
  defp escript do
    [main_module: :merge_records,
     language: :erlang,
     app: nil]
  end

  defp deps do
    [{:couchbeam, github: "benoitc/couchbeam", tag: "1.3.1"}]
  end
end
