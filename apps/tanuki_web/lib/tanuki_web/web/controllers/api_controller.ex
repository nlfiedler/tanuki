defmodule TanukiWeb.Web.ApiController do
  use TanukiWeb.Web, :controller
  require Logger

  def tags(conn, _params) do
    results = TanukiBackend.all_tags()
    tags = for row <- results do
      %{
        tag: :couchbeam_doc.get_value("key", row),
        count: :couchbeam_doc.get_value("value", row)
      }
    end
    json conn, tags
  end
end
