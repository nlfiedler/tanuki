defmodule TanukiWeb.PageController do
  use TanukiWeb.Web, :controller

  plug :fetch_tags

  def index(conn, params) do
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
    |> render(:index)
  end

  def detail(conn, params) do
    row_id = to_charlist(params["id"])
    {:ok, document} = :tanuki_backend.fetch_document(row_id)
    sha256 = to_string(:couchbeam_doc.get_value(<<"sha256">>, document))
    filename = to_string(:couchbeam_doc.get_value(<<"file_name">>, document))
    filesize = to_string(:couchbeam_doc.get_value(<<"file_size">>, document))
    location = to_string(:couchbeam_doc.get_value(<<"location">>, document))
    topic = to_string(:couchbeam_doc.get_value(<<"topic">>, document))
    tags = for t <- :couchbeam_doc.get_value(<<"tags">>, document), do: to_string(t)
    datetime_list = :tanuki_backend.get_best_date(document)
    datetime_str = :tanuki_backend.date_list_to_string(datetime_list)
    asset_info = %{
      :fname => filename,
      :size => filesize,
      :datetime => datetime_str,
      :sha => sha256,
      :topic => topic,
      :location => location,
      :tags => tags
    }
    conn
    |> assign(:asset_info, asset_info)
    |> render(:detail)
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

  def asset(conn, params) do
    sha256 = to_charlist(params["id"])
    filepath = to_string(:tanuki_backend.checksum_to_asset_path(sha256))
    mimetype = case :tanuki_backend.by_checksum(sha256) do
        [] ->
          "application/octet-stream"
        [doc|_t] ->
          to_string(:couchbeam_doc.get_value(<<"value">>, doc))
    end
    # The Etag is just the checksum, which is already the best possible
    # value for this asset.
    etag = params["id"]
    # Use the send_file function to avoid loading the entire asset into
    # memory, since it could be an enormous video.
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", etag)
    |> send_file(200, filepath)
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
    row_id = to_string(:couchbeam_doc.get_value(<<"id">>, row))
    values = :couchbeam_doc.get_value(<<"value">>, row)
    date_string = :tanuki_backend.date_list_to_string(hd(values), :date_only)
    filename = to_string(hd(tl(values)))
    checksum = to_string(hd(tl(tl(values))))
    %{
      :id => row_id,
      :fname => filename,
      :date => date_string,
      :sha => checksum
    }
  end
end
