defmodule TanukiWeb.PageController do
  use TanukiWeb.Web, :controller

  # pages of thumbnails are limited to 3 by 6
  @page_size 18

  plug :fetch_tags

  def index(conn, params) do
    # order matters here
    conn
    |> tag_selection(params)
    |> tag_unselection(params)
    |> page_selection(params)
    |> assign_assets_and_tags()
    |> assign_pagination_data()
    |> render(:index)
  end

  def detail(conn, params) do
    row_id = to_charlist(params["id"])
    {:ok, document} = :tanuki_backend.fetch_document(row_id)
    asset_info = read_doc(document)
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

  def preview(conn, params) do
    sha256 = to_charlist(params["id"])
    binary = :tanuki_backend.generate_thumbnail(sha256, :preview)
    etag = :crypto.hash(:sha, binary) |> Base.encode16 |> String.downcase
    conn
    |> put_resp_content_type("image/jpeg")
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
          to_string(:couchbeam_doc.get_value("value", doc))
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

  def edit(conn, params) do
    row_id = to_charlist(params["id"])
    {:ok, document} = :tanuki_backend.fetch_document(row_id)
    asset_info = read_doc(document)
    conn
    |> assign(:asset_info, asset_info)
    |> render(:edit)
  end

  def update(conn, params) do
    row_id = to_charlist(params["id"])
    # Field values are strings, which in Elixir are already binaries, so no
    # need to convert before sending to couchbeam.
    {:ok, document} = :tanuki_backend.fetch_document(row_id)
    newdoc = :couchbeam_doc.set_value("location", params["location"], document)
    newdoc = :couchbeam_doc.set_value("topic", params["topic"], newdoc)
    tags = for t <- String.split(params["tags"], ","), do: String.trim(t)
    newdoc = :couchbeam_doc.set_value("tags", tags, newdoc)
    {:ok, updated} = :tanuki_backend.update_document(newdoc)
    asset_info = read_doc(updated)
    conn
    |> assign(:asset_info, asset_info)
    |> render(:detail)
  end

  defp read_doc(document) do
    row_id = to_string(:couchbeam_doc.get_id(document))
    sha256 = to_string(:couchbeam_doc.get_value("sha256", document))
    filename = to_string(:couchbeam_doc.get_value("file_name", document))
    filesize = to_string(:couchbeam_doc.get_value("file_size", document))
    location = to_string(:couchbeam_doc.get_value("location", document))
    topic = to_string(:couchbeam_doc.get_value("topic", document))
    tags = for t <- :couchbeam_doc.get_value("tags", document), do: to_string(t)
    datetime_list = :tanuki_backend.get_best_date(document)
    datetime_str = :tanuki_backend.date_list_to_string(datetime_list)
    %{
      :id => row_id,
      :fname => filename,
      :size => filesize,
      :datetime => datetime_str,
      :sha => sha256,
      :topic => topic,
      :location => location,
      :tags => tags
    }
  end

  # Fetches the known tags and assigns the list to the connection as
  # `tags`, to be rendered by the "keys.html" template.
  defp fetch_tags(conn, _opts) do
    results = :tanuki_backend.all_tags()
    tags = for row <- results, do: :couchbeam_doc.get_value("key", row)
    assign(conn, :tags, tags)
  end

  defp tag_selection(conn, params) do
    if Map.has_key?(params, "add_tag") do
      new_tag = params["add_tag"]
      selected_tags = get_selected_tags(conn)
      if new_tag in selected_tags do
        conn
      else
        conn
        |> put_session(:selected_tags, selected_tags ++ [new_tag])
        # changing the tags must reset the current page
        |> put_session(:curr_page, nil)
      end
    else
      conn
    end
  end

  defp tag_unselection(conn, params) do
    if Map.has_key?(params, "del_tag") do
      selected_tags = get_selected_tags(conn) -- [params["del_tag"]]
      conn
      |> put_session(:selected_tags, selected_tags)
      # changing the tags must reset the current page
      |> put_session(:curr_page, nil)
    else
      conn
    end
  end

  defp get_selected_tags(conn) do
    selected_tags = get_session(conn, :selected_tags)
    if selected_tags == nil do
      []
    else
      selected_tags
    end
  end

  defp page_selection(conn, params) do
    if Map.has_key?(params, "page") do
      put_session(conn, :curr_page, params["page"])
    else
      conn
    end
  end

  defp assign_pagination_data(conn) do
    tag_info = conn.assigns[:tag_info]
    if length(tag_info) > @page_size do
      curr_page = get_current_page(conn)
      page_count = round(Float.ceil(length(tag_info) / @page_size))
      #
      # Compute the lower and upper page numbers, clipping to whatever is
      # possible given the number of pages we have. Hence the combination
      # of 'cond', 'min', and 'max' below.
      #
      desired_lower = curr_page - 10
      desired_upper = curr_page + 9
      {lower, upper} = cond do
        desired_lower <= 1 ->
          {2, min(desired_upper + abs(desired_lower), page_count - 1)}
        desired_upper >= page_count ->
          {max(desired_lower - (desired_upper - page_count), 2), page_count - 1}
        true -> {desired_lower, desired_upper}
      end
      pages = Enum.into(Range.new(lower, upper), [])
      page_data = %{
        :first_page => 1,
        :pages => pages,
        :last_page => page_count
      }
      start = (curr_page - 1) * @page_size
      tag_info = Enum.slice(tag_info, start, @page_size)
      conn
      |> assign(:page_data, page_data)
      |> assign(:curr_page, curr_page)
      |> assign(:tag_info, tag_info)
    else
      assign(conn, :page_data, %{})
    end
  end

  defp get_current_page(conn) do
    curr_page = get_session(conn, :curr_page)
    if curr_page == nil do
      1
    else
      {value, _Rest} = Integer.parse(curr_page)
      value
    end
  end

  defp assign_assets_and_tags(conn) do
    selected_tags = get_selected_tags(conn)
    tags_as_charlist = for tag <- selected_tags, do: to_charlist(tag)
    rows = :tanuki_backend.by_tags(tags_as_charlist, &asset_row_sorter/2)
    tag_info = for row <- rows, do: build_asset_info(row)
    conn
    |> assign(:selected_tags, selected_tags)
    |> assign(:tag_info, tag_info)
  end

  defp asset_row_sorter(a, b) do
    # The date is the first value in the list of "value" in the row in the
    # 'by_tag' CouchDB view. By default sort newer assets before older.
    a_date = hd(:couchbeam_doc.get_value("value", a))
    b_date = hd(:couchbeam_doc.get_value("value", b))
    a_date >= b_date
  end

  defp build_asset_info(row) do
    row_id = to_string(:couchbeam_doc.get_value("id", row))
    values = :couchbeam_doc.get_value("value", row)
    # values is a list of [date, file_name, sha256], where 'date' is exif,
    # file, or import date in that preferred order. The date value itself
    # is a list of integers (e.g. [2014, 7, 4, 12, 1] ~> "2014/7/4 12:01").
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
