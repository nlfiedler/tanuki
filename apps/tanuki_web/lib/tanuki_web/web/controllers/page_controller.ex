defmodule TanukiWeb.Web.PageController do
  use TanukiWeb.Web, :controller
  require Logger

  def index(conn, _params) do
    render(conn, :index)
  end

  def thumbnail(conn, params) do
    checksum = params["id"]
    {:ok, binary, mimetype} = TanukiBackend.retrieve_thumbnail(checksum)
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", checksum <> ".thumb")
    |> send_resp(200, binary)
  end

  def preview(conn, params) do
    checksum = params["id"]
    {:ok, binary, mimetype} = TanukiBackend.generate_thumbnail(checksum, :preview)
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", checksum <> ".preview")
    |> send_resp(200, binary)
  end

  def asset(conn, params) do
    checksum = params["id"]
    filepath = TanukiBackend.checksum_to_asset_path(checksum)
    mimetype = case TanukiBackend.by_checksum(checksum) do
      [] -> "application/octet-stream"
      [doc|_t] -> :couchbeam_doc.get_value("value", doc)
    end
    conn
    |> put_resp_content_type(mimetype)
    |> put_resp_header("etag", checksum <> ".asset")
    |> send_asset(filepath)
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
        # this asset already exists, simply forward to the edit page
        :couchbeam_doc.get_value("id", doc)
    end
    redirect conn, to: "/assets/#{doc_id}/edit"
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
