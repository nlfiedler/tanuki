defmodule TanukiBackend do
  @moduledoc """

  Interface to the CouchDB database for tanuki.

  """
  require Logger
  require Record

  # Mnesia record for storing image thumbnails keyed by checksum.
  Record.defrecord :thumbnails, [sha256: nil, binary: nil]

  # Mnesia record for tracking the age of individual thumbnails.
  Record.defrecord :thumbnail_dates, [timestamp: nil, sha256: nil]

  # Mnesia record for limiting the number of cached thumbnails.
  Record.defrecord :thumbnail_counter, [id: 0, ver: 1]

  # Mnesia record for caching "by_tags" queries.
  Record.defrecord :by_tags_cache, [key: nil, results: nil]

  # Mnesia record for caching "by_date" queries.
  Record.defrecord :by_date_cache, [key: nil, results: nil]

  @mnesia_tables [
    :thumbnails,
    :thumbnail_dates,
    :thumbnail_counter,
    :by_tags_cache,
    :by_date_cache
  ]

  @doc """

  Initialize the mnesia tables for caching thumbnails. Should be called by
  the application at startup.

  """
  def init_schema() do
    nodes = [:erlang.node()]
    ensure_schema(nodes)
    ensure_mnesia(nodes)
    :ok = :mnesia.wait_for_tables(@mnesia_tables, 5000)
  end

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

  Retrieves all documents with a given checksum, as couchbeam view results.
  Result includes the id and mimetype fields.

  """
  @spec by_checksum(checksum) :: [any()] when checksum: String.t
  def by_checksum(checksum) when is_binary(checksum) do
    GenServer.call(TanukiDatabase, {:by_checksum, checksum})
  end

  @doc """

  Retrieves all documents with the given tags, as couchbeam view results.
  Only those documents containing all of the given tags will be returned.
  Ordering is non-deterministic.

  Results are cached based on the tags, so any changes in the database will
  not be reflected by repeated queries for the same set of tags. However,
  this serves repeated requests very well, such as for allowing efficient
  pagination of many results.

  """
  @spec by_tags(tags) :: [any()] when tags: String.t
  def by_tags(tags) when is_list(tags) do
    by_tags(tags, fn(_a, _b) -> true end)
  end

  @doc """

  Like by_tags/1 with the results sorted according to the given function.

  """
  @spec by_tags(tags, sort_fn) :: [any()] when tags: [String.t], sort_fn: (any(), any() -> boolean())
  def by_tags(tags, sort_fn) when is_list(tags) do
    # Produce a key based on the requested tags and the sorting function
    # such that if either change, the key changes.
    key = {Enum.join(Enum.sort(tags), ""), sort_fn}
    tags_cache_query_fn = fn() ->
      case :mnesia.read(:by_tags_cache, key) do
        [] ->
          Logger.info("cache miss for #{inspect key}")
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
          unsorted_results = :lists.usort(fn(a, b) ->
            id_a = :couchbeam_doc.get_value("id", a)
            id_b = :couchbeam_doc.get_value("id", b)
            id_a <= id_b
          end, matching_rows)
          # Now sort the rows according to the given sorting function.
          results = Enum.sort(unsorted_results, sort_fn)
          # Save the results in mnesia to avoid doing the same work again
          # if a repeated request is made, replacing any previous values.
          case :mnesia.first(:by_tags_cache) do
            :'$end_of_table' -> :ok
            first_key -> :ok = :mnesia.delete({:by_tags_cache, first_key})
          end
          :ok = :mnesia.write(by_tags_cache(key: key, results: results))
          results
        [by_tags_cache(results: rows)] ->
          Logger.info("cache hit for #{inspect key}")
          rows
      end
    end
    :mnesia.activity(:transaction, tags_cache_query_fn)
  end

  @doc """

  Retrieves all documents whose most relevant date is within the given
  year. The date used will be exif_date, or file_date, or import_date, in
  that order. Results are as from couchbeam_view:fetch/3.

  """
  @spec by_date(year) :: [any()] when year: integer()
  def by_date(year) when is_integer(year) do
    key = year
    year_cache_query_fn = fn() ->
      case :mnesia.read(:by_date_cache, key) do
        [] ->
          Logger.info("cache miss for #{inspect key}")
          start_date = [year, 0, 0, 0, 0]
          end_date = [year + 1, 0, 0, 0, 0]
          results = GenServer.call(TanukiDatabase, {:by_date, start_date, end_date})
          # Save the results in mnesia to avoid doing the same work again
          # if a repeated request is made, replacing any previous values.
          case :mnesia.first(:by_date_cache) do
            :'$end_of_table' -> :ok
            first_key -> :ok = :mnesia.delete({:by_date_cache, first_key})
          end
          :ok = :mnesia.write(by_date_cache(key: key, results: results))
          results
        [by_date_cache(results: rows)] ->
          Logger.info("cache hit for #{inspect key}")
          rows
      end
    end
    :mnesia.activity(:transaction, year_cache_query_fn)
  end

  @doc """

  Retrieves all documents whose most relevant date is within the given
  month. The date used will be exif_date, or file_date, or import_date, in
  that order. Results are as from couchbeam_view:fetch/3.

  """
  @spec by_date(year, month) :: [any()] when year: integer(), month: integer()
  def by_date(year, month) when is_integer(year) and is_integer(month) do
    start_date = [year, month, 0, 0, 0]
    end_date = [year, month + 1, 0, 0, 0]
    GenServer.call(TanukiDatabase, {:by_date, start_date, end_date})
  end

  @doc """

  Converts a date-list (list of integers representing a date) of the form
  [<year>, <month>, <day>, <hour>, <minutes>] to a string. For example, the
  list [2014, 7, 4, 12, 1] would become "2014/7/4 12:01".

  """
  @spec date_list_to_string(date_list) :: String.t when date_list: [integer()]
  def date_list_to_string(date_list) do
    to_string(List.flatten(:io_lib.format("~B/~B/~B ~B:~2.10.0B", date_list)))
  end

  @doc """

  Converts a date-list (list of integers representing a date) of the
  form [2014, 7, 4, 12, 1] to a string, with only the date: 2014/7/4.

  """
  @spec date_list_to_string(date_list, :date_only) :: String.t when date_list: [integer()]
  def date_list_to_string(date_list, :date_only) do
    to_string(List.flatten(:io_lib.format("~B/~B/~B", Enum.slice(date_list, 0, 3))))
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

  Either retrieve the thumbnail produced earlier, or generate one now and
  cache for later use. Returns {:ok, binary, mimetype}, where mimetype will
  always be "image/jpeg".

  """
  @spec retrieve_thumbnail(String.t) :: {:ok, binary(), String.t}
  def retrieve_thumbnail(checksum) do
    # look for thumbnail cached in mnesia, producing and storing, if needed
    thumbnail_fn = fn() ->
      case :mnesia.read(:thumbnails, checksum) do
        [thumbnails(binary: binary)] ->
          # record the time this thumbnail was accessed
          Logger.info("cache hit for thumbnail #{checksum}")
          now = DateTime.to_unix(DateTime.utc_now())
          :ok = :mnesia.write(thumbnail_dates(timestamp: now, sha256: checksum))
          binary
        [] ->
          Logger.info("cache miss for thumbnail #{checksum}")
          # producing a thumbnail in a transaction is not ideal...
          binary = generate_thumbnail(checksum, :thumbnail)
          :ok = :mnesia.write(thumbnails(sha256: checksum, binary: binary))
          now = DateTime.to_unix(DateTime.utc_now())
          :ok = :mnesia.write(thumbnail_dates(timestamp: now, sha256: checksum))
          # update the count of thumbnails currently cached
          count = :mnesia.dirty_update_counter(:thumbnail_counter, :id, 1)
          # prune oldest record if we reached our limit
          if count > 1000 do
            # this finds the oldest thumbnail, where age is determined by
            # either when it was generated or when it was last retrieved
            Logger.info("discarding oldest cached thumbnail")
            oldest_key = :mnesia.first(:thumbnail_dates)
            [thumbnail_dates(sha256: oldest)] = :mnesia.read(:thumbnail_dates, oldest_key)
            :ok = :mnesia.delete({:thumbnails, oldest})
            :ok = :mnesia.delete({:thumbnail_dates, oldest_key})
            :mnesia.dirty_update_counter(:thumbnail_counter, :id, -1)
          end
          binary
      end
    end
    binary = :mnesia.activity(:transaction, thumbnail_fn)
    {:ok, binary, "image/jpeg"}
  end

  @doc """

  Produce a jpeg thumbnail of the image file designated by the given SHA256
  checksum. Two convenient sizes are available, either :thumbnail which
  resizes the image to a box of 240 by 240 pixels, or :preview, which
  resizes the image to a box of 640 by 640 pixels. Or you can provide an
  integer value of your own choosing.

  """
  @spec generate_thumbnail(String.t, :thumbnail | :preview | integer()) :: binary()
  def generate_thumbnail(checksum, :thumbnail) do
    generate_thumbnail(checksum, 240)
  end

  def generate_thumbnail(checksum, :preview) do
    generate_thumbnail(checksum, 640)
  end

  def generate_thumbnail(checksum, size) when is_integer(size) do
    source_file = checksum_to_asset_path(checksum)
    # Avoid attempting to generate a thumbnail for large files, which are
    # very likely not image files at all (e.g. videos). The value of 10MB
    # was arrived at by examining a collection of images and videos that
    # represent typical usage. That is, most images are less than 10MB and
    # most videos are over 10MB; the overlap is acceptable, such that some
    # large images will not get thumbnails, and some videos will be pulled
    # into memory only to result in an error.
    case File.stat(source_file) do
      {:ok, fstat} ->
        if fstat.size < 10485760 do
          case File.read(source_file) do
            {:ok, image_data} ->
              case :emagick_rs.image_fit(image_data, size, size) do
                {:ok, resized} -> resized
                {:error, reason} ->
                  Logger.warn("unable to resize asset #{checksum}: #{reason}")
                  broken_image_placeholder()
              end
            {:error, reason} ->
              Logger.warn("unable to read asset file #{source_file}: #{reason}")
              broken_image_placeholder()
          end
        else
          broken_image_placeholder()
        end
      {:error, reason} ->
        Logger.warn("unable to stat asset file #{source_file}: #{reason}")
        broken_image_placeholder()
    end
  end

  @doc """

  Return the image data for the broken image placeholder thumbnail.

  """
  @spec broken_image_placeholder() :: binary()
  def broken_image_placeholder() do
    priv_dir = :code.priv_dir(:tanuki_backend)
    image_path = Path.join(priv_dir, "images/broken_image.jpg")
    File.read!(image_path)
  end

  @doc """

  Extract the most accurate date from the given document. The precedence
  is EXIF original date, followed by file date, followed by import date.
  The date is the format stored in the database (a list of integers).

  """
  @spec get_best_date(any()) :: list() | :none
  def get_best_date(doc) do
    case get_field_value("exif_date", doc) do
      :none ->
        case get_field_value("file_date", doc) do
          :none -> get_field_value("import_date", doc)
          date -> date
        end
      date -> date
    end
  end

  @doc """

  Extract the value of the named field in the given document, or :none if
  the value is :undefined or :null.

  """
  @spec get_field_value(String.t, any()) :: any() | :none
  def get_field_value(field, document) do
    case :couchbeam_doc.get_value(field, document) do
      :undefined -> :none
      :null -> :none
      value -> value
    end
  end

  @doc """

  Ensure the schema and our table is installed in mnesia.

  """
  def ensure_schema(nodes) do
    # Create the schema if it does not exist
    if :mnesia.system_info(:schema_version) == {0, 0} do
      :ok = :mnesia.create_schema(nodes)
    end
    ensure_tables = fn() ->
      tables = :mnesia.system_info(:tables)
      if not Enum.member?(tables, :thumbnails) do
        {:atomic, :ok} = :mnesia.create_table(:thumbnails, [
          # first field of the record is the table key
          {:attributes, Keyword.keys(thumbnails(thumbnails()))}
        ])
      end
      if not Enum.member?(tables, :thumbnail_dates) do
        {:atomic, :ok} = :mnesia.create_table(:thumbnail_dates, [
          # first field of the record is the table key
          {:attributes, Keyword.keys(thumbnail_dates(thumbnail_dates()))},
          {:type, :ordered_set}
        ])
      end
      if not Enum.member?(tables, :thumbnail_counter) do
        {:atomic, :ok} = :mnesia.create_table(:thumbnail_counter, [
          {:attributes, Keyword.keys(thumbnail_counter(thumbnail_counter()))}
        ])
      end
      if not Enum.member?(tables, :by_tags_cache) do
        {:atomic, :ok} = :mnesia.create_table(:by_tags_cache, [
          {:attributes, Keyword.keys(by_tags_cache(by_tags_cache()))}
        ])
      end
      if not Enum.member?(tables, :by_date_cache) do
        {:atomic, :ok} = :mnesia.create_table(:by_date_cache, [
          {:attributes, Keyword.keys(by_date_cache(by_date_cache()))}
        ])
      end
    end
    # Create our table if it does not exist
    if :mnesia.system_info(:is_running) == :no do
      :rpc.multicall(nodes, :application, :start, [:mnesia])
      ensure_tables.()
      :rpc.multicall(nodes, :application, :stop, [:mnesia])
    else
      ensure_tables.()
    end
  end

  # Ensure the mnesia application is running.
  defp ensure_mnesia(nodes) do
    if :mnesia.system_info(:is_running) == :no do
      :rpc.multicall(nodes, :application, :start, [:mnesia])
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
      {:ok, db} = :couchbeam.open_or_create_db(server, db_name, [])
      :ok = install_designs(db)
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

    defp install_designs(db) do
      # Look for .json files in our private views directory and insert them
      # directly into CouchDB. They are presumed to be our design documents
      # and thus needed for general operation.
      views_dir = Path.join(:code.priv_dir(:tanuki_backend), "views")
      insert_doc_fn = fn(filename) ->
        filepath = Path.join(views_dir, filename)
        json = :couchbeam_ejson.decode(File.read!(filepath))
        doc_id = :couchbeam_doc.get_id(json)
        if :couchbeam.doc_exists(db, doc_id) do
          {:ok, doc} = :couchbeam.open_doc(db, doc_id)
          old_views = :couchbeam_doc.get_value("views", doc)
          new_views = :couchbeam_doc.get_value("views", json)
          if old_views != new_views do
            doc = :couchbeam_doc.set_value("views", new_views, doc)
            {:ok, _doc1} = :couchbeam.save_doc(db, doc)
          end
        else
          {:ok, _doc1} = :couchbeam.save_doc(db, json)
        end
      end
      json_selector_fn = fn(name) -> Path.extname(name) == ".json" end
      json_files = Enum.filter(File.ls!(views_dir), json_selector_fn)
      for filename <- json_files, do: insert_doc_fn.(filename)
      :ok
    end
  end
end
