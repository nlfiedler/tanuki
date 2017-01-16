defmodule TanukiWeb.PageController do
  use TanukiWeb.Web, :controller

  # pages of thumbnails are limited to 3 by 6
  @page_size 18

  def index(conn, params) do
    # order matters here
    conn
    |> load_all_tags()
    |> load_all_years()
    |> tag_selection(params)
    |> tag_unselection(params)
    |> page_selection(params)
    |> assign_tag_info()
    |> assign_pagination_data(:tag_info)
    |> render(:index)
  end

  def detail(conn, params) do
    {:ok, document} = TanukiBackend.fetch_document(params["id"])
    asset_info = read_doc(document)
    conn
    |> assign(:asset_info, asset_info)
    |> render(:detail)
  end

  def thumbnail(conn, params) do
    {:ok, binary, mimetype} = TanukiBackend.retrieve_thumbnail(params["id"])
    etag = Base.encode16(:crypto.hash(:sha, binary), case: :lower)
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", etag)
    |> send_resp(200, binary)
  end

  def preview(conn, params) do
    binary = TanukiBackend.generate_thumbnail(params["id"], :preview)
    etag = Base.encode16(:crypto.hash(:sha, binary), case: :lower)
    conn
    |> put_resp_content_type("image/jpeg")
    |> put_resp_header("etag", etag)
    |> send_resp(200, binary)
  end

  def asset(conn, params) do
    filepath = to_string(TanukiBackend.checksum_to_asset_path(params["id"]))
    mimetype = case TanukiBackend.by_checksum(params["id"]) do
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
    {:ok, document} = TanukiBackend.fetch_document(params["id"])
    asset_info = read_doc(document)
    conn
    |> assign(:asset_info, asset_info)
    |> render(:edit)
  end

  def update(conn, params) do
    # Field values are strings, which in Elixir are already binaries, so no
    # need to convert before sending to couchbeam.
    {:ok, document} = TanukiBackend.fetch_document(params["id"])
    newdoc = :couchbeam_doc.set_value("location", params["location"], document)
    newdoc = :couchbeam_doc.set_value("caption", params["caption"], newdoc)
    tags = for t <- String.split(params["tags"], ","), do: String.trim(t)
    newdoc = :couchbeam_doc.set_value("tags", tags, newdoc)
    {:ok, updated} = TanukiBackend.update_document(newdoc)
    asset_info = read_doc(updated)
    conn
    |> assign(:asset_info, asset_info)
    |> render(:detail)
  end

  def year(conn, params) do
    {year, _rest} = Integer.parse(params["id"])
    conn
    |> load_all_years()
    |> year_selection(year)
    |> page_selection(params)
    |> assign_year_info(year)
    |> assign_pagination_data(:year_info)
    |> assign(:year, year)
    |> render(:year)
  end

  defp year_selection(conn, year) do
    selected_year = get_session(conn, :selected_year)
    # when not set, the session value will be nil
    if selected_year != year do
      conn
      |> put_session(:selected_year, year)
      # reset the current page if the year selection changed (even from nil)
      |> put_session(:curr_page, nil)
    else
      conn
    end
  end

  defp read_doc(document) do
    row_id = to_string(:couchbeam_doc.get_id(document))
    sha256 = to_string(:couchbeam_doc.get_value("sha256", document))
    filename = to_string(:couchbeam_doc.get_value("file_name", document))
    filesize = to_string(:couchbeam_doc.get_value("file_size", document))
    location = to_string(:couchbeam_doc.get_value("location", document))
    caption = to_string(:couchbeam_doc.get_value("caption", document))
    tags = for t <- :couchbeam_doc.get_value("tags", document), do: to_string(t)
    datetime_list = TanukiBackend.get_best_date(document)
    datetime_str = TanukiBackend.date_list_to_string(datetime_list)
    %{
      :id => row_id,
      :fname => filename,
      :size => filesize,
      :datetime => datetime_str,
      :sha => sha256,
      :caption => caption,
      :location => location,
      :tags => tags
    }
  end

  # Fetches the known tags and assigns the list to the connection as
  # `tags`, to be rendered by the "tags.html" template.
  defp load_all_tags(conn) do
    results = TanukiBackend.all_tags()
    tags = for row <- results, do: :couchbeam_doc.get_value("key", row)
    # the number of documents with that tag...
    # count = :couchbeam_doc.get_value("value", row)
    assign(conn, :tags, tags)
  end

  # Fetches the known years and assigns the list to the connection as
  # `years`, to be rendered by the "years.html" template.
  defp load_all_years(conn) do
    results = TanukiBackend.all_years()
    years = for row <- results, do: :couchbeam_doc.get_value("key", row)
    # the number of documents with that year...
    # count = :couchbeam_doc.get_value("value", row)
    assign(conn, :years, years)
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

  defp assign_pagination_data(conn, keyname) do
    data_set = conn.assigns[keyname]
    if length(data_set) > @page_size do
      curr_page = get_current_page(conn)
      page_count = round(Float.ceil(length(data_set) / @page_size))
      pages = if page_count > 2 do
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
        Enum.into(Range.new(lower, upper), [])
      else
        # Special case for just two pages, no need for the convoluted math
        # that ends up producing goofy page links.
        []
      end
      page_data = %{
        :first_page => 1,
        :pages => pages,
        :last_page => page_count
      }
      start = (curr_page - 1) * @page_size
      data_set = Enum.slice(data_set, start, @page_size)
      conn
      |> assign(:page_data, page_data)
      |> assign(:curr_page, curr_page)
      |> assign(keyname, data_set)
    else
      assign(conn, :page_data, %{})
    end
  end

  defp get_current_page(conn) do
    curr_page = get_session(conn, :curr_page)
    if curr_page == nil do
      1
    else
      {value, _rest} = Integer.parse(curr_page)
      value
    end
  end

  defp assign_tag_info(conn) do
    selected_tags = get_selected_tags(conn)
    rows = TanukiBackend.by_tags(selected_tags, &tag_row_sorter/2)
    tag_info = for row <- rows, do: build_tag_info(row)
    conn
    |> assign(:selected_tags, selected_tags)
    |> assign(:tag_info, tag_info)
  end

  defp tag_row_sorter(a, b) do
    # The date is the first value in the list of "value" in the row in the
    # 'by_tag' CouchDB view. By default sort newer assets before older.
    a_date = hd(:couchbeam_doc.get_value("value", a))
    b_date = hd(:couchbeam_doc.get_value("value", b))
    a_date >= b_date
  end

  defp build_tag_info(row) do
    row_id = to_string(:couchbeam_doc.get_value("id", row))
    values = :couchbeam_doc.get_value("value", row)
    # values is a list of [date, file_name, sha256], where 'date' is exif,
    # file, or import date in that preferred order. The date value itself
    # is a list of integers (e.g. [2014, 7, 4, 12, 1] ~> "2014/7/4 12:01").
    date_string = TanukiBackend.date_list_to_string(hd(values), :date_only)
    filename = to_string(hd(tl(values)))
    checksum = to_string(hd(tl(tl(values))))
    %{
      :id => row_id,
      :fname => filename,
      :date => date_string,
      :sha => checksum
    }
  end

  defp assign_year_info(conn, year) do
    rows = TanukiBackend.by_date(year)
    year_info = for row <- rows, do: build_year_info(row)
    assign(conn, :year_info, year_info)
  end

  defp build_year_info(row) do
    row_id = :couchbeam_doc.get_value("id", row)
    key = :couchbeam_doc.get_value("key", row)
    date_string = TanukiBackend.date_list_to_string(key, :date_only)
    values = :couchbeam_doc.get_value("value", row)
    # values is a list of [file_name, sha256]
    filename = to_string(hd(values))
    checksum = to_string(hd(tl(values)))
    %{
      :id => row_id,
      :fname => filename,
      :date => date_string,
      :sha => checksum
    }
  end
end
