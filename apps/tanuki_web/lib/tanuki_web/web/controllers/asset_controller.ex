defmodule TanukiWeb.Web.AssetController do
  use TanukiWeb.Web, :controller
  require Logger

  def index(conn, params) do
    # TODO: support "location" parameter to filter by location (exact)
    # TODO: support "year" parameter to filter by year (exact)
    # TODO: support "order" parameter to sort the results
    tags = params["tags"]
    cond do
      is_nil(tags) ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(400, ~s({"error": "missing required tags argument"}))
      not is_list(tags) ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(400, ~s({"error": "tags must be list"}))
      true ->
        rows = TanukiBackend.by_tags(tags, &by_tags_sorter/2)
        tag_info = for row <- rows, do: build_asset_info(row)
        # count is the number of _all_ matching results
        count = length(tag_info)
        # handle pagination with certain defaults and bounds
        page_size = bounded_int_value(params["page_size"], 10, 1, 100)
        page_limit = max(1, round(count / page_size))
        page = bounded_int_value(params["page"], 1, 1, page_limit)
        start = (page - 1) * page_size
        results = Enum.slice(tag_info, start, page_size)
        json conn, %{:assets => results, :count => count}
    end
  end

  def show(conn, params) do
    case TanukiBackend.fetch_document(params["id"]) do
      {:error, :not_found} ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(404, ~s({"error": "no such asset"}))
      {:error, reason} ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(400, ~s({"error": "#{reason}"}))
      {:ok, document} ->
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
        duration = get_duration(mimetype, sha256)
        json conn, %{
          :id => row_id,
          :filename => filename,
          :size => filesize,
          :mimetype => mimetype,
          :datetime => datetime_str,
          :checksum => sha256,
          :caption => caption,
          :location => location,
          :duration => duration,
          :tags => tags
        }
    end
  end

  def create(conn, params) do
    plug_upload = params["asset"]
    {:ok, checksum} = TanukiIncoming.compute_checksum(plug_upload.path)
    # check if an asset with this checksum already exists
    doc_id = case TanukiBackend.by_checksum(checksum) do
      [] ->
        original_date = TanukiIncoming.get_original_date(plug_upload.path)
        fstat = File.stat!(plug_upload.path)
        {:ok, import_date} = TanukiIncoming.time_tuple_to_list(:calendar.universal_time())
        doc_values = {[
          {"original_date", original_date},
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
        # this asset already exists
        :couchbeam_doc.get_value("id", doc)
    end
    json conn, %{:status => "success", :id => doc_id}
  end

  def update(conn, params) do
    case TanukiBackend.fetch_document(params["id"]) do
      {:error, :not_found} ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(404, ~s({"error": "no such asset"}))
      {:error, reason} ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(400, ~s({"error": "#{reason}"}))
      {:ok, document} ->
        newdoc = :couchbeam_doc.set_value("location", params["location"], document)
        newdoc = :couchbeam_doc.set_value("caption", params["caption"], newdoc)
        tags = for t <- String.split(params["tags"], ","), do: String.trim(t)
        newdoc = :couchbeam_doc.set_value("tags", Enum.uniq(Enum.sort(tags)), newdoc)
        newdoc = if String.length(params["user_date"]) > 0 do
          # the expected format of the optional date string is yyyy/mm/dd
          parts = String.split(params["user_date"], "/")
          day = String.to_integer(hd(tl(tl(parts))))
          month = String.to_integer(hd(tl(parts)))
          year = String.to_integer(hd(parts))
          # add the given date to the time from the best available date/time
          datetime_list = TanukiBackend.get_best_date(document)
          new_dt_list = [year, month, day] ++ Enum.slice(datetime_list, 3, 2)
          :couchbeam_doc.set_value("user_date", new_dt_list, newdoc)
        else
          newdoc
        end
        {:ok, _updated} = TanukiBackend.update_document(newdoc)
        json conn, %{:status => "success"}
    end
  end

  defp get_duration(mimetype, checksum) do
    if String.starts_with?(mimetype, "video/") do
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
  end

  defp by_tags_sorter(a, b) do
    # The date is the first value in the list of "value" in the row in the
    # 'by_tag' CouchDB view. By default sort newer assets before older.
    a_date = hd(:couchbeam_doc.get_value("value", a))
    b_date = hd(:couchbeam_doc.get_value("value", b))
    a_date >= b_date
  end

  defp build_asset_info(row) do
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
      :filename => filename,
      :date => date_string,
      :checksum => checksum
    }
  end

  @doc """

  Return an integer given the input value. If value is nil, then return
  default. If value is an integer, return that, bounded by the minimum and
  maximum values. If value is a string, parse as an integer and ensure it
  falls within the minimum and maximum bounds.

  """
  def bounded_int_value(value, default, minimum, maximum) do
    v = cond do
      is_nil(value) -> default
      is_integer(value) -> value
      is_binary(value) ->
        case Integer.parse(value) do
          {i, ""} -> i
          _ -> default
        end
    end
    min(max(v, minimum), maximum)
  end
end
