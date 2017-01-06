defmodule TanukiWeb.PageController do
  use TanukiWeb.Web, :controller

  plug :fetch_tags

  def index(conn, _params) do
    render conn, :index
  end

  def tagged(conn, params) do
    conn = if Map.has_key?(params, "add_tag") do
      new_tag = params["add_tag"]
      selected_tags = get_selected_tags(conn)
      if new_tag in selected_tags do
        conn
      else
        put_session(conn, :selected_tags, selected_tags ++ [new_tag])
      end
    else
      conn
    end
    conn = if Map.has_key?(params, "del_tag") do
      selected_tags = get_selected_tags(conn) -- [params["del_tag"]]
      put_session(conn, :selected_tags, selected_tags)
    else
      conn
    end
    selected_tags = get_selected_tags(conn)
    tag_info = collect_asset_info(selected_tags)
    conn
    |> assign(:selected_tags, selected_tags)
    |> assign(:tag_info, tag_info)
    |> render(:tagged)
  end

  def thumbnail(conn, params) do
    sha256 = to_charlist(params["id"])
    {:ok, binary, mimetype} = :tanuki_backend.retrieve_thumbnail(sha256)
    etag = :crypto.hash(:sha, binary) |> Base.encode16 |> String.downcase
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", etag)
    |> send_resp(200, binary)
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

  defp collect_asset_info(tags) do
    tags_as_charlist = for tag <- tags, do: to_charlist(tag)
    rows = :tanuki_backend.by_tags(tags_as_charlist)
    for row <- rows, do: build_asset_info(row)
  end

  defp build_asset_info(row) do
    # TODO: probably need the id for showing the details?
    # row_id = to_string(:couchbeam_doc.get_value(<<"id">>, row))
    values = :couchbeam_doc.get_value(<<"value">>, row)
    date_string = :tanuki_backend.date_list_to_string(hd(values), :date_only)
    filename = to_string(hd(tl(values)))
    checksum = to_string(hd(tl(tl(values))))
    %{
      # :id => row_id,
      :fname => filename,
      :date => date_string,
      :sha => checksum
    }
  end
end
