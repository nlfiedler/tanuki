defmodule TanukiBackend do
  @moduledoc """

  Interface to the CouchDB database for tanuki.

  """
  require Logger

  @doc """

  Retrieve the document in CouchDB identified by the given internal value
  (what would appear in the '_id' field in the document).

  """
  @spec fetch_document(String.t) :: {:ok, any()} | {:error, any()}
  def fetch_document(doc_id) do
    GenServer.call(TanukiDatabase, {:fetch_document, doc_id})
  end

  @doc """

  Update the given document in the database, returning the updated document
  (as returned from CouchDB).

  """
  @spec update_document(any()) :: {:ok, any()} | {:error, any()}
  def update_document(doc) do
    GenServer.call(TanukiDatabase, {:update_document, doc})
  end

  @doc """

  Return the total number of assets stored in the database.

  """
  def count_assets() do
    GenServer.call(TanukiDatabase, :count_assets)
  end

  @doc """

  Retrieves all known tags as `couchbeam` view results.

  """
  @spec all_tags() :: [any()]
  def all_tags() do
    GenServer.call(TanukiDatabase, :all_tags)
  end

  @doc """

  Retrieves all known years as `couchbeam` view results.

  """
  @spec all_years() :: [any()]
  def all_years() do
    GenServer.call(TanukiDatabase, :all_years)
  end

  @doc """

  Retrieves all known locations as `couchbeam` view results.

  """
  @spec all_locations() :: [any()]
  def all_locations() do
    GenServer.call(TanukiDatabase, :all_locations)
  end

  @doc """

  Retrieves all documents with a given checksum, as couchbeam view results.
  Result includes the id and mimetype fields.

  """
  @spec by_checksum(checksum) :: [any()] when checksum: String.t
  def by_checksum(checksum) when is_binary(checksum) do
    GenServer.call(TanukiDatabase, {:by_checksum, checksum})
  end

  @doc """

  Query assets by tags, or years, or locations, or any combination of
  those. Each argument is a list of values on which to select assets.

  """
  def query(nil, nil, locations) do
    query_by_location_fn = fn(location) ->
      GenServer.call(TanukiDatabase, {:by_location, location})
    end
    Enum.flat_map(locations, query_by_location_fn)
  end

  def query(nil, years, nil) do
    query_by_year_fn = fn(year) ->
      start_date = [year, 0, 0, 0, 0]
      end_date = [year + 1, 0, 0, 0, 0]
      GenServer.call(TanukiDatabase, {:by_date, start_date, end_date})
    end
    Enum.flat_map(years, query_by_year_fn)
  end

  def query(tags, nil, nil) do
    by_tags(tags)
  end

  def query(nil, years, locations) do
    by_years = query(nil, years, nil)
    Enum.filter(by_years, &filter_by_location(locations, &1))
  end

  def query(tags, years, nil) do
    rows_by_tags = by_tags(tags)
    Enum.filter(rows_by_tags, &filter_by_year(years, &1))
  end

  def query(tags, nil, locations) do
    rows_by_tags = by_tags(tags)
    Enum.filter(rows_by_tags, &filter_by_location(locations, &1))
  end

  def query(tags, years, locations) do
    rows_by_tags = by_tags(tags)
    filtered_by_years = Enum.filter(rows_by_tags, &filter_by_year(years, &1))
    Enum.filter(filtered_by_years, &filter_by_location(locations, &1))
  end

  defp filter_by_year(years, row) do
    values = :couchbeam_doc.get_value("value", row)
    year = hd(hd(values))
    Enum.any?(years, &(year == &1))
  end

  defp filter_by_location(locations, row) do
    values = :couchbeam_doc.get_value("value", row)
    location = Enum.at(values, 3)
    Enum.any?(locations, &(location == &1))
  end

  @doc """

  Retrieves all documents with the given tags, as couchbeam view results.
  Only those documents containing all of the given tags will be returned.
  Ordering is non-deterministic.

  """
  @spec by_tags(tags) :: [any()] when tags: String.t
  def by_tags(tags) when is_list(tags) do
    all_rows = GenServer.call(TanukiDatabase, {:by_tags, tags})
    # Reduce the results to those that have all of the given tags.
    tag_counts = List.foldl(all_rows, %{}, fn(row, acc_in) ->
      docid = :couchbeam_doc.get_value("id", row)
      count = Map.get(acc_in, docid, 0)
      Map.put(acc_in, docid, count + 1)
    end)
    matching_rows = Enum.filter(all_rows, fn(row) ->
      docid = :couchbeam_doc.get_value("id", row)
      Map.get(tag_counts, docid) == length(tags)
    end)
    # Remove the duplicate rows by sorting on the document identifier
    # in a unique fashion.
    :lists.usort(fn(a, b) ->
      id_a = :couchbeam_doc.get_value("id", a)
      id_b = :couchbeam_doc.get_value("id", b)
      id_a <= id_b
    end, matching_rows)
  end

  @doc """

  Retrieves all documents whose most relevant date is within the given
  year. The date used will be user_date, original_date, or file_date, or
  import_date, in that order. Results are as from couchbeam_view:fetch/3.

  """
  @spec by_date(year) :: [any()] when year: integer()
  def by_date(year) when is_integer(year) do
    start_date = [year, 0, 0, 0, 0]
    end_date = [year + 1, 0, 0, 0, 0]
    GenServer.call(TanukiDatabase, {:by_date, start_date, end_date})
  end

  @doc """

  Retrieves all documents whose most relevant date is within the given
  month. The date used will be user_date, original_date, or file_date, or
  import_date, in that order. Results are as from couchbeam_view:fetch/3.

  """
  @spec by_date(year, month) :: [any()] when year: integer(), month: integer()
  def by_date(year, month) when is_integer(year) and is_integer(month) do
    start_date = [year, month, 0, 0, 0]
    end_date = [year, month + 1, 0, 0, 0]
    GenServer.call(TanukiDatabase, {:by_date, start_date, end_date})
  end

  @doc """

  Retrieves all documents whose most relevant location is within the given
  year. The date used will be user_date, original_date, or file_date, or
  import_date, in that order. Results are as from couchbeam_view:fetch/3.

  """
  @spec by_location(String.t) :: [any()]
  def by_location(location) do
    GenServer.call(TanukiDatabase, {:by_location, location})
  end

  @doc """

  Converts a date-list (list of integers representing a date) of the form
  [<year>, <month>, <day>, <hour>, <minutes>] to a string. For example, the
  list [2014, 7, 4, 12, 1] would become "2014-07-04 12:01".

  If the parameter is :undefined or :null, returns the empty string.

  """
  @spec date_list_to_string(date_list) :: String.t when date_list: [integer()]
  def date_list_to_string(empty) when empty == :undefined or empty == :null do
    ""
  end

  def date_list_to_string(date_list) do
    to_string(List.flatten(:io_lib.format("~4.10.0B-~2.10.0B-~2.10.0B ~2.10.0B:~2.10.0B", date_list)))
  end

  @doc """

  Converts a date-list (list of integers representing a date) of the
  form [2014, 7, 4, 12, 1] to a string, with only the date: 2014-07-04.

  """
  @spec date_list_to_string(date_list, :date_only) :: String.t when date_list: [integer()]
  def date_list_to_string(date_list, :date_only) do
    to_string(List.flatten(:io_lib.format("~4.10.0B-~2.10.0B-~2.10.0B", Enum.slice(date_list, 0, 3))))
  end

  @doc """

  For a given SHA256 checksum, return the path to the asset.

  """
  @spec checksum_to_asset_path(String.t) :: String.t
  def checksum_to_asset_path(checksum) do
    assets_dir = Application.get_env(:tanuki_backend, :assets_dir)
    part1 = String.slice(checksum, 0, 2)
    part2 = String.slice(checksum, 2, 2)
    # 64 is the length of a SHA256 in hexadecimal form
    part3 = String.slice(checksum, 4, 64)
    Path.join([assets_dir, part1, part2, part3])
  end

  @doc """

  Produce a jpeg thumbnail of the image file designated by the given SHA256
  checksum and return the path to the generated thumbnail. Two convenient
  sizes are available, either :thumbnail which resizes the image to a box
  of 240 by 240 pixels, or :preview, which resizes the image to a box of
  640 by 640 pixels.

  """
  @spec generate_thumbnail(String.t, :thumbnail | :preview) :: String.t()
  def generate_thumbnail(checksum, size) do
    infile = checksum_to_asset_path(checksum)
    if File.exists?(infile) do
      pixels = image_size(size)
      outfile = checksum_to_thumbs_path(checksum, pixels)
      if File.exists?(outfile) do
        Logger.info("cache hit for thumbnail #{checksum}")
        outfile
      else
        Logger.info("cache miss for thumbnail #{checksum}")
        File.mkdir_p!(Path.dirname(outfile))
        mimetype = case TanukiBackend.by_checksum(checksum) do
          [] -> "application/octet-stream"
          [doc|_t] -> :couchbeam_doc.get_value("value", doc)
        end
        cond do
          String.starts_with?(mimetype, "video/") ->
            cmd = [
              "ffmpeg", "-loglevel", "quiet", "-n",
              "-i", infile, "-vframes", "1", "-an",
              "-filter:v", "scale=w=#{pixels}:h=#{pixels}:force_original_aspect_ratio=decrease",
              outfile
            ]
            port = Port.open({:spawn, Enum.join(cmd, " ")}, [:exit_status])
            case wait_for_port(port) do
              {:ok, 0} -> outfile
              {:ok, _n} ->
                Logger.warn("unable to resize asset #{checksum}")
                broken_image_placeholder()
            end
          String.starts_with?(mimetype, "image/") ->
            dimensions = "'#{pixels}x#{pixels}>'"
            cmd = ["convert", infile, "-thumbnail", dimensions, "-unsharp", "0x.5", outfile]
            port = Port.open({:spawn, Enum.join(cmd, " ")}, [:exit_status])
            case wait_for_port(port) do
              {:ok, 0} -> outfile
              {:ok, _n} ->
                Logger.warn("unable to resize asset #{checksum}")
                broken_image_placeholder()
            end
          true ->
            broken_image_placeholder()
        end
      end
    else
      Logger.warn("no such asset #{checksum}")
      broken_image_placeholder()
    end
  end

  @doc """

  Return the pixel size that corresponds to the logical image size.

  """
  def image_size(:thumbnail), do: 240
  def image_size(:preview), do: 640

  @doc """

  Return the path to the broken image placeholder thumbnail.

  """
  @spec broken_image_placeholder() :: binary()
  def broken_image_placeholder() do
    priv_dir = :code.priv_dir(:tanuki_backend)
    Path.join([priv_dir, "images", "broken_image.jpg"])
  end

  @doc """

  For a given SHA256 checksum, return the path to the thumbnail, which may
  not yet have been generated. The filename extension will always be .jpg
  because thumbnails are always JPEG images.

  """
  @spec checksum_to_thumbs_path(String.t, integer()) :: String.t
  def checksum_to_thumbs_path(checksum, size) when is_integer(size) do
    thumbs_dir = Application.get_env(:tanuki_backend, :thumbnails_dir)
    # By adding the extension, we signal to the convert command that the
    # desired thumbnail image format should be JPEG.
    extension = ".jpg"
    # For now all thumbnails will be in one directory, with a level that
    # separates the "thumbnail" images from the "preview" images.
    Path.join([thumbs_dir, Integer.to_string(size), checksum <> extension])
  end

  # Remove enough images to stay within the limit.
  def prune_old_thumbnails(size, limit) do
    dirname = Path.dirname(checksum_to_thumbs_path("cafebabe", size))
    filetimes = for name <- File.ls!(dirname) do
      fpath = Path.join(dirname, name)
      atime = File.stat!(fpath).atime
      {fpath, atime}
    end
    if length(filetimes) > limit do
      sorted = List.keysort(filetimes, 1)
      oldest = Enum.slice(sorted, 0, length(sorted) - limit)
      for {fpath, _atime} <- oldest do
        Logger.info("pruning cached thumbnail #{fpath}")
        File.rm!(fpath)
      end
    end
  end

  @doc """

  Wait for the given port to complete and return the exit code in the form
  of {:ok, status}. If the port experiences an error, returns {:error,
  reason}.

  """
  @spec wait_for_port(port()) :: {:ok, integer()} | {:error, any()}
  def wait_for_port(port) do
    receive do
      {^port, {:exit_status, status}} ->
        ensure_port_closed(port)
        {:ok, status}
      {^port, {:data, _data}} ->
        Logger.info("output from port ignored")
        wait_for_port(port)
      {:EXIT, ^port, reason} ->
        Logger.error("port #{port} had an error: #{reason}")
        {:error, reason}
    end
  end

  # Ensure that the given Port has been properly closed. Does nothing if
  # the port is not open.
  defp ensure_port_closed(port) do
    unless Port.info(port) == nil do
      Port.close(port)
    end
  end

  @doc """

  Extract the most accurate date from the given document. The precedence
  is EXIF original date, followed by file date, followed by import date.
  The date is the format stored in the database (a list of integers).

  """
  @spec get_best_date(any()) :: list() | nil
  def get_best_date(doc) do
    case get_field_value("user_date", doc) do
      nil ->
        case get_field_value("original_date", doc) do
          nil ->
            case get_field_value("file_date", doc) do
              nil -> get_field_value("import_date", doc)
              date -> date
            end
          date -> date
        end
      date -> date
    end
  end

  @doc """

  Extract the value of the named field in the given document, or nil if
  the value is :undefined or :null.

  """
  @spec get_field_value(String.t, any()) :: any() | nil
  def get_field_value(field, document) do
    case :couchbeam_doc.get_value(field, document) do
      :undefined -> nil
      :null -> nil
      value -> value
    end
  end

  defmodule Server do
    use GenServer

    defmodule State do
      defstruct [:server, :database]
    end

    def init([]) do
      url = Application.get_env(:tanuki_backend, :couchdb_url)
      opts = Application.get_env(:tanuki_backend, :couchdb_opts)
      db_name = Application.get_env(:tanuki_backend, :database)
      server = :couchbeam.server_connection(url, opts)
      {:ok, db} = :couchbeam.open_or_create_db(server, db_name)
      :ok = install_designs(db)
      # set up a timer to thin the thumbnails directory every 5 minutes
      :timer.apply_interval(300000, :gen_server, :cast, [TanukiDatabase, :thin_thumbnails])
      {:ok, %State{server: server, database: db}}
    end

    def start_link(state, opts \\ []) do
      GenServer.start_link(__MODULE__, state, opts)
    end

    def handle_call({:fetch_document, doc_id}, _from, state) do
      {:reply, :couchbeam.open_doc(state.database, doc_id), state}
    end

    def handle_call({:update_document, doc}, _from, state) do
      {:reply, :couchbeam.save_doc(state.database, doc), state}
    end

    def handle_call(:count_assets, _from, state) do
      count = :couchbeam_view.count(state.database)
      {:reply, count, state}
    end

    def handle_call(:all_tags, _from, state) do
      options = [{:group_level, 1}]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "tags"}, options)
      {:reply, rows, state}
    end

    def handle_call(:all_years, _from, state) do
      options = [{:group_level, 1}]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "years"}, options)
      {:reply, rows, state}
    end

    def handle_call(:all_locations, _from, state) do
      options = [{:group_level, 1}]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "locations"}, options)
      {:reply, rows, state}
    end

    def handle_call({:by_checksum, checksum}, _from, state) do
      options = [{:key, checksum}]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "by_checksum"}, options)
      {:reply, rows, state}
    end

    def handle_call({:by_tags, tags}, _from, state) do
      options = [{:keys, tags}]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "by_tag"}, options)
      {:reply, rows, state}
    end

    def handle_call({:by_date, start_date, end_date}, _from, state) do
      options = [
        {:start_key, start_date},
        {:end_key, end_date}
      ]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "by_date"}, options)
      {:reply, rows, state}
    end

    def handle_call({:by_location, location}, _from, state) do
      options = [{:key, location}]
      {:ok, rows} = :couchbeam_view.fetch(state.database, {"assets", "by_location"}, options)
      {:reply, rows, state}
    end

    def handle_cast(:thin_thumbnails, state) do
      thumbnails_limit = Application.get_env(:tanuki_backend, :thumbnails_limit)
      TanukiBackend.prune_old_thumbnails(TanukiBackend.image_size(:thumbnail), thumbnails_limit)
      previews_limit = Application.get_env(:tanuki_backend, :previews_limit)
      TanukiBackend.prune_old_thumbnails(TanukiBackend.image_size(:preview), previews_limit)
      {:noreply, state}
    end

    @doc """

    Locate the JavaScript view definitions in the priv directory and load
    them into CouchDB, if they differ from what is already there.

    """
    @spec install_designs(any()) :: :ok
    def install_designs(db) do
      # Look for .js files in our private views directory and insert them
      # into CouchDB as views for performing queries.
      views_dir = Path.join(:code.priv_dir(:tanuki_backend), "views")
      doc_id = "_design/assets"
      js_selector_fn = fn(fname) -> Path.extname(fname) == ".js" end
      js_files = Enum.filter(File.ls!(views_dir), js_selector_fn)
      view_tuples = for fname <- js_files, do: read_view_js(Path.join(views_dir, fname))
      if :couchbeam.doc_exists(db, doc_id) do
        {:ok, doc} = :couchbeam.open_doc(db, doc_id)
        {old_views} = :couchbeam_doc.get_value("views", doc)
        if :lists.keysort(1, old_views) != :lists.keysort(1, view_tuples) do
          doc = :couchbeam_doc.set_value("views", {view_tuples}, doc)
          {:ok, _doc1} = :couchbeam.save_doc(db, doc)
          Logger.info("updated _design/assets document")
        end
      else
        doc = {[
          {"_id", doc_id},
          {"language", "javascript"},
          {"views", {view_tuples}}
        ]}
        {:ok, _doc1} = :couchbeam.save_doc(db, doc)
        Logger.info("created _design/assets document")
      end
      :ok
    end

    @doc """

    Read the named JavaScript file and produce a tuple suitable for an
    entry in the "views" field of a CouchDB design document. If the file
    contains a comment line "//!reduce:" then the value after the colon
    will be the value for the "reduce" function of the view.

    For example: {"file": {"map": "code...", "reduce": "_count"}}

    """
    @spec read_view_js(String.t) :: list()
    def read_view_js(filename) do
      text = File.open!(filename, [:read, :utf8], &IO.read(&1, :all))
      lines = Enum.map(String.split(text, "\n", trim: true), &String.trim(&1))
      {comments, code} = Enum.partition(lines, &String.starts_with?(&1, "//"))
      result = [{"map", Enum.join(code, " ")}]
      result = case Enum.find(comments, &String.starts_with?(&1, "//!reduce:")) do
        nil -> result
        reduce ->
          trimmed = String.trim(String.replace_leading(reduce, "//!reduce:", ""))
          result ++ [{"reduce", trimmed}]
      end
      {Path.rootname(Path.basename(filename)), {result}}
    end
  end
end
