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

  def locations(conn, _params) do
    results = TanukiBackend.all_locations()
    locations = for row <- results do
      %{
        location: :couchbeam_doc.get_value("key", row),
        count: :couchbeam_doc.get_value("value", row)
      }
    end
    json conn, locations
  end

  def years(conn, _params) do
    results = TanukiBackend.all_years()
    years = for row <- results do
      %{
        year: :couchbeam_doc.get_value("key", row),
        count: :couchbeam_doc.get_value("value", row)
      }
    end
    json conn, years
  end
end
