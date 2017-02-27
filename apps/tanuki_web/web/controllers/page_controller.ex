defmodule TanukiWeb.PageController do
  use TanukiWeb.Web, :controller
  require Logger

  # pages of thumbnails are limited to 3 by 6
  @page_size 18

  def index(conn, params) do
    # order matters here
    conn
    |> load_all_tags()
    |> load_all_years()
    |> load_all_locations()
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
    |> assign_duration(asset_info)
    |> render(:detail)
  end

  def thumbnail(conn, params) do
    checksum = params["id"]
    filepath = TanukiBackend.generate_thumbnail(checksum, :thumbnail)
    etag = checksum <> ".thumb"
    # Use send_file and let the OS and browser cache the binary.
    conn
    |> put_resp_content_type("image/jpeg")
    |> put_resp_header("etag", etag)
    |> send_file(200, filepath)
  end

  def preview(conn, params) do
    checksum = params["id"]
    filepath = TanukiBackend.generate_thumbnail(checksum, :preview)
    etag = checksum <> ".preview"
    # Use send_file and let the OS and browser cache the binary.
    conn
    |> put_resp_content_type("image/jpeg")
    |> put_resp_header("etag", etag)
    |> send_file(200, filepath)
  end

  def asset(conn, params) do
    checksum = params["id"]
    filepath = TanukiBackend.checksum_to_asset_path(checksum)
    mimetype = case TanukiBackend.by_checksum(params["id"]) do
      [] -> "application/octet-stream"
      [doc|_t] -> :couchbeam_doc.get_value("value", doc)
    end
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", checksum <> ".asset")
    |> send_asset(filepath)
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
    newdoc = :couchbeam_doc.set_value("tags", Enum.uniq(Enum.sort(tags)), newdoc)
    newdoc = if String.length(params["user_date"]) > 0 do
      # the expected format of the optional date string is mm/dd/yyyy
      parts = String.split(params["user_date"], "/")
      year = String.to_integer(hd(tl(tl(parts))))
      day = String.to_integer(hd(tl(parts)))
      month = String.to_integer(hd(parts))
      # add the given date to the time from the best available date/time
      datetime_list = TanukiBackend.get_best_date(document)
      new_dt_list = [year, month, day] ++ Enum.slice(datetime_list, 3, 2)
      :couchbeam_doc.set_value("user_date", new_dt_list, newdoc)
    else
      newdoc
    end
    {:ok, updated} = TanukiBackend.update_document(newdoc)
    asset_info = read_doc(updated)
    conn
    |> assign(:asset_info, asset_info)
    |> render(:detail)
  end

  def upload(conn, _params) do
    render(conn, :upload)
  end

  def import(conn, params) do
    plug_upload = params["asset"]
    {:ok, checksum} = TanukiIncoming.compute_checksum(plug_upload.path)
    # check if an asset with this checksum already exists
    doc_id = case TanukiBackend.by_checksum(checksum) do
      [] ->
        exif_date = TanukiIncoming.get_original_exif_date(plug_upload.path)
        fstat = File.stat!(plug_upload.path)
        import_date = TanukiIncoming.time_tuple_to_list(:calendar.universal_time())
        doc_values = {[
          {"exif_date", exif_date},
          {"file_name", plug_upload.filename},
          {"file_size", fstat.size},
          {"import_date", import_date},
          {"mimetype", plug_upload.content_type},
          {"sha256", checksum},
          # everything generally assumes the tags field is not undefined
          {"tags", []}
        ]}
        {:ok, new_doc} = TanukiBackend.update_document(doc_values)
        TanukiIncoming.store_asset(plug_upload.path, checksum)
        :couchbeam_doc.get_id(new_doc)
      [doc|_t] ->
        # this asset already exists, simply forward to the edit page
        :couchbeam_doc.get_value("id", doc)
    end
    redirect conn, to: "/asset/#{doc_id}/edit"
  end

  def year(conn, params) do
    year = String.to_integer(params["id"])
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

  def location(conn, params) do
    location = params["id"]
    conn
    |> load_all_locations()
    |> location_selection(location)
    |> page_selection(params)
    |> assign_location_info(location)
    |> assign_pagination_data(:location_info)
    |> assign(:location, location)
    |> render(:location)
  end

  defp location_selection(conn, location) do
    selected_location = get_session(conn, :selected_location)
    # when not set, the session value will be nil
    if selected_location != location do
      conn
      |> put_session(:selected_location, location)
      # reset the current page if the location selection changed (even from nil)
      |> put_session(:curr_page, nil)
    else
      conn
    end
  end

  defp read_doc(document) do
    row_id = :couchbeam_doc.get_id(document)
    sha256 = :couchbeam_doc.get_value("sha256", document)
    filename = :couchbeam_doc.get_value("file_name", document)
    filesize = :couchbeam_doc.get_value("file_size", document)
    location = TanukiBackend.get_field_value("location", document)
    caption = TanukiBackend.get_field_value("caption", document)
    mimetype = :couchbeam_doc.get_value("mimetype", document)
    tags = :couchbeam_doc.get_value("tags", document)
    datetime_list = TanukiBackend.get_best_date(document)
    datetime_str = TanukiBackend.date_list_to_string(datetime_list)
    %{
      :id => row_id,
      :fname => filename,
      :size => filesize,
      :mimetype => mimetype,
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

  # Fetches the known locations and assigns the list to the connection as
  # `locations`, to be rendered by the "locations.html" template.
  defp load_all_locations(conn) do
    results = TanukiBackend.all_locations()
    locations = for row <- results, do: :couchbeam_doc.get_value("key", row)
    # the number of documents with that location...
    # count = :couchbeam_doc.get_value("value", row)
    assign(conn, :locations, locations)
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
      String.to_integer(curr_page)
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
    row_id = :couchbeam_doc.get_value("id", row)
    values = :couchbeam_doc.get_value("value", row)
    # values is a list of [date, file_name, sha256], where 'date' is exif,
    # file, or import date in that preferred order. The date value itself
    # is a list of integers (e.g. [2014, 7, 4, 12, 1] ~> "2014/7/4 12:01").
    date_string = TanukiBackend.date_list_to_string(hd(values), :date_only)
    filename = hd(tl(values))
    checksum = hd(tl(tl(values)))
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
    filename = hd(values)
    checksum = hd(tl(values))
    %{
      :id => row_id,
      :fname => filename,
      :date => date_string,
      :sha => checksum
    }
  end

  defp assign_location_info(conn, location) do
    rows = TanukiBackend.by_location(location)
    location_info = for row <- rows, do: build_location_info(row)
    assign(conn, :location_info, location_info)
  end

  defp build_location_info(row) do
    row_id = :couchbeam_doc.get_value("id", row)
    values = :couchbeam_doc.get_value("value", row)
    # values is a list of [date, file_name, sha256], where 'date' is user,
    # exif, file, or import date in that preferred order. The date value
    # itself is a list of integers (e.g. [2014, 7, 4, 12, 1]).
    date_string = TanukiBackend.date_list_to_string(hd(values), :date_only)
    filename = hd(tl(values))
    checksum = hd(tl(tl(values)))
    %{
      :id => row_id,
      :fname => filename,
      :date => date_string,
      :sha => checksum
    }
  end

  defp assign_duration(conn, asset_info) do
    duration = if String.starts_with?(asset_info[:mimetype], "video/") do
      checksum = asset_info[:sha]
      filepath = TanukiBackend.checksum_to_asset_path(checksum)
      ffprobe_args = [
        "-loglevel", "quiet", "-show_entries",
        "format=duration", "-of", "default=noprint_wrappers=1:nokey=1",
        filepath
      ]
      case System.cmd("ffprobe", ffprobe_args) do
        {output, 0} ->
          round(String.to_float(String.trim(output)))
        {output, code} ->
          Logger.warn("ffprobe exited non-zero (#{code}): #{output}")
          nil
      end
    else
      nil
    end
    assign(conn, :duration, duration)
  end

  defp send_asset(conn, filepath) do
    # Send the asset file back as requested by the client, either with a
    # specific content range, or the entire file all at once. This is
    # required for video playback to work in Safari. This does not handle
    # multi-range specs but that seems to have no practical impact.
    if List.keymember?(conn.req_headers, "range", 0) do
      fstat = File.stat!(filepath)
      {first_byte, last_byte} = case List.keyfind(conn.req_headers, "range", 0) do
        {"range", "bytes=-" <> suffix_len} ->
          {max(0, fstat.size - String.to_integer(suffix_len)), fstat.size - 1}
        {"range", "bytes=" <> byte_range_spec} ->
          if String.ends_with?(byte_range_spec, "-") do
            {String.split(byte_range_spec, "-") |> hd |> String.to_integer, fstat.size - 1}
          else
            range = for n <- String.split(byte_range_spec, "-"), do: String.to_integer(n)
            {hd(range), hd(tl(range))}
          end
      end
      length = last_byte - first_byte + 1
      conn
      |> put_resp_header("content-range", "bytes #{first_byte}-#{last_byte}/#{fstat.size}")
      |> send_file(206, filepath, first_byte, length)
    else
      send_file(conn, 200, filepath)
    end
  end
end
