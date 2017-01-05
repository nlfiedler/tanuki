defmodule TanukiWeb.PageController do
  use TanukiWeb.Web, :controller

  plug :fetch_tags

  def index(conn, _params) do
    render conn, :index
  end

  def tagged(conn, params) do
    conn = if Map.has_key?(params, "add_tag") do
      selected_tags = get_selected_tags(conn) ++ [Map.get(params, "add_tag")]
      put_session(conn, :selected_tags, selected_tags)
    else
      conn
    end
    conn = if Map.has_key?(params, "del_tag") do
      selected_tags = get_selected_tags(conn) -- [Map.get(params, "del_tag")]
      put_session(conn, :selected_tags, selected_tags)
    else
      conn
    end
    selected_tags = get_selected_tags(conn)
    conn
    |> assign(:selected_tags, selected_tags)
    |> render(:tagged)
  end

  # Fetches the known tags and assigns the list to the connection as
  # `tags`, to be rendered by the "keys.html" template.
  defp fetch_tags(conn, _opts) do
    results = :tanuki_backend.all_tags()
    tags = for row <- results, do: :couchbeam_doc.get_value(<<"key">>, row)
    assign(conn, :tags, tags)
  end

  defp get_selected_tags(conn) do
    selected_tags = get_session(conn, :selected_tags)
    if selected_tags == nil do
      []
    else
      selected_tags
    end
  end
end
