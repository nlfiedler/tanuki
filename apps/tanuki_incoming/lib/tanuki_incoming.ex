defmodule TanukiIncoming do
  @moduledoc """

  Server for incorporating incoming assets into the system.

  """
  require Logger

  defmodule Server do
    use GenServer

    defmodule State do
      defstruct [:server, :database]
    end

    def start_link(state, opts \\ []) do
      GenServer.start_link(__MODULE__, state, opts)
    end

    def init([]) do
      url = Application.get_env(:tanuki_incoming, :couchdb_url)
      opts = Application.get_env(:tanuki_incoming, :couchdb_opts)
      db_name = Application.get_env(:tanuki_incoming, :database)
      server = :couchbeam.server_connection(url, opts)
      case :couchbeam.open_or_create_db(server, db_name) do
        {:ok, db} ->
          {:ok, _tref} = :timer.apply_interval(
            1000 * 60 * 60, :gen_server, :cast, [TanukiIncoming, :process])
          {:ok, %State{
            server: server,
            database: db
          }}
        {:error, reason} ->
          raise "Cannot connect to CouchDB: #{reason}"
      end
    end

    # Primarily for testing, so the test code can block until a response is
    # received. Returns {:ok, count} where count is the number of new
    # assets added to the database.
    def handle_call(:process_now, _from, state) do
      # All directories are old enough as we want to process them now.
      filter_fn = fn(_path) -> true end
      incoming_dir = Application.get_env(:tanuki_incoming, :incoming_dir)
      before_count = :couchbeam_view.count(state.database)
      TanukiIncoming.process_incoming(incoming_dir, state.database, filter_fn)
      after_count = :couchbeam_view.count(state.database)
      {:reply, {:ok, after_count - before_count}, state}
    end

    def handle_cast(:process, state) do
      # Filter function to ensure the given directory is at least an hour old.
      filter_fun = fn(path) ->
        nowsecs = DateTime.to_unix(DateTime.utc_now())
        fstat = File.stat!(path, time: :posix)
        (nowsecs - fstat.ctime) > 3600
      end
      incoming_dir = Application.get_env(:tanuki_incoming, :incoming_dir)
      TanukiIncoming.process_incoming(incoming_dir, state.database, filter_fun)
      {:noreply, state}
    end
  end

  @doc """

  Process the assets in the incoming directory, storing them in the blob
  store area and inserting records in the database.

  """
  def process_incoming(incoming, db, filter_fn) do
    Logger.info("incoming asset processing commencing...")
    # Get the names of the directories in the incoming path.
    filepaths = for name <- File.ls!(incoming), do: Path.join(incoming, name)
    directories = Enum.filter(filepaths, &File.dir?/1)
    Logger.info("considering the following directories: #{directories}")
    # Check if the given directory is sufficiently old.
    old_enough = Enum.filter(directories, filter_fn)
    if length(old_enough) == 0 do
      Logger.info("no sufficiently old directories found at this time")
    else
      Enum.each(old_enough, fn(path) -> process_path(path, db) end)
    end
    :ok
  end

  @doc """

  Import the assets found with the named path.

  """
  def process_path(path, db) do
    {tags, location} = convert_path_to_details(path)
    Logger.info("processing assest with tags #{tags}, location #{location}")
    :ok = delete_extraneous_files(path)
    process_fn = fn(name) ->
      filepath = Path.join(path, name)
      fstat = File.stat!(filepath)
      if fstat.type == :regular do
        {:ok, checksum} = compute_checksum(filepath)
        # Either insert or update a document in the database.
        case find_document(db, checksum) do
          nil -> create_document(db, filepath, tags, location, checksum)
          doc_id -> update_document(db, doc_id, tags, location)
        end
        # Move the asset into place, or remove it if duplicate.
        store_asset(filepath, checksum)
      else
        Logger.warn("ignoring non-file entry #{name}")
      end
    end
    Enum.each(File.ls!(path), process_fn)
    Logger.info("done with #{path}")
    case File.rmdir(path) do
      :ok -> :ok;
      {:error, reason} ->
        Logger.error("failed to remove #{path}: #{reason}")
    end
  end

  @doc """

  Convert the path to a set of tags and a location. Location, if any,
  starts with an at sign (@) and goes to the end of the path. Tags must be
  separated by underscore (_). Any underscores in location are replaced
  with spaces.

  """
  def convert_path_to_details(path) do
    asset_folder = String.downcase(Path.basename(path))
    [tags | tail] = String.split(asset_folder, "@")
    location = case tail do
      [] -> nil
      [l | _] -> String.replace(l, "_", " ")
    end
    tag_list = String.split(tags, "_")
    non_empty_tags = Enum.filter(tag_list, fn(tag) -> String.length(tag) > 0 end)
    {non_empty_tags, location}
  end

  @doc """

  Compute the SHA256 checksum for the named file, returning the result as a
  string of hexadecimal digits. Processes the file in chunks to avoid
  reading possibly very large files.

  """
  def compute_checksum(filename) do
    handle = File.open!(filename, [:read])
    context = :crypto.hash_init(:sha256)
    case compute_checksum(handle, context) do
      {:ok, digest} -> {:ok, Base.encode16(digest, case: :lower)}
      r -> r
    end
  end

  # Helper function that recursively computes the SHA256 of the opened file
  # in 64KB chunks. The file will be closed upon successful completion.
  defp compute_checksum(handle, context) do
    case IO.binread(handle, 65536) do
      {:error, reason} -> {:error, reason}
      :eof ->
        case File.close(handle) do
          :ok -> {:ok, :crypto.hash_final(context)}
          rr -> rr
        end
      data ->
        context = :crypto.hash_update(context, data)
        compute_checksum(handle, context)
    end
  end

  @doc """

  Determine if the name file is a JPEG encoded image by scanning the first
  and last two bytes.

  """
  def jpeg?(filepath) do
    # JPEG files start with 0xFFD8 and end with 0xFFD9.
    file = File.open!(filepath, [:read])
    first_bytes = IO.binread(file, 2)
    {:ok, _np} = :file.position(file, {:eof, -2})
    last_bytes = IO.binread(file, 2)
    File.close(file)
    <<255, 216>> == first_bytes and <<255, 217>> == last_bytes
  end

  @doc """

  Return true if the image does not require orientation correction, and
  false otherwise. If the asset is not an image, or the orientation field
  is missing, then true is returned (i.e. there is nothing that can be
  done).

  """
  @spec correct_orientation?(String.t) :: boolean()
  def correct_orientation?(filepath) do
    # We can only correct the orientation for images, which for the time
    # being, are assumed to be JPEG images.
    case jpeg?(filepath) do
      true ->
        case :exif.read(to_charlist(filepath)) do
          {:error, reason} ->
            Logger.warn("unable to read EXIF data in #{filepath}: #{reason}")
            true
          {:ok, exif_data} ->
            case :dict.find(:orientation, exif_data) do
              {:ok, "Top-left"} ->
                true
              {:ok, _n} ->
                false
              :error ->
                Logger.info("no orientation setting in #{filepath}")
                true
            end
        end
      false -> true
    end
  end

  @doc """

  Attempt to read the original datetime from the EXIF tags, returning :null
  if not available, or the date time as a list of integers. Uses :null since
  the value will be inserted directly into CouchDB.

  """
  def get_original_exif_date(filepath) do
    # Exif data only exists in JPEG files, and no use loading a large video
    # file into memory only to find it isn't an image at all.
    if jpeg?(filepath) do
      case :exif.read(to_charlist(filepath)) do
        {:error, reason} ->
          Logger.error("unable to read EXIF data from #{filepath}, #{reason}")
          :null
        {:ok, exif_data} ->
          case :dict.find(:date_time_original, exif_data) do
            {:ok, original_date} ->
              time_tuple_to_list(date_parse(original_date))
            :error ->
              Logger.info("no original date available from #{filepath}")
              :null
          end
      end
    else
      :null
    end
  end

  @doc """

  Create a new document in the CouchDB database.

  """
  def create_document(db, fullpath, tags, location, checksum) do
    filename = Path.basename(fullpath)
    Logger.info("creating document for #{filename}")
    fstat = File.stat!(fullpath)
    {:ok, uid_details} = :epwd_rs.getpwuid(fstat.uid)
    file_owner = to_string(Keyword.get(uid_details, :pw_name))
    doc = {[
      {"exif_date", get_original_exif_date(fullpath)},
      {"file_date", time_tuple_to_list(fstat.mtime)},
      {"file_name", filename},
      {"file_owner", file_owner},
      {"file_size", fstat.size},
      {"import_date", time_tuple_to_list(:calendar.universal_time())},
      {"location", location},
      {"mimetype", hd(:mimetypes.filename(String.downcase(filename)))},
      {"sha256", checksum},
      {"tags", Enum.sort(tags)}
    ]}
    # Fail fast if insertion failed, so we do not then move the asset out
    # of the incoming directory and fail to process it properly.
    {:ok, new_doc} = :couchbeam.save_doc(db, doc)
    {id, rev} = :couchbeam_doc.get_idrev(new_doc)
    Logger.info("#{filename} => id=#{id}, rev=#{rev}")
    {:ok, id}
  end

  @doc """

  Merge the given tags with existing document's tags.
  If missing location field, set to Location argument.

  """
  def update_document(db, doc_id, tags, location) do
    Logger.info("updating document #{doc_id}")
    {:ok, doc} = :couchbeam.open_doc(db, doc_id)
    # set the location field in the document, if not already set
    doc = if TanukiBackend.get_field_value("location", doc) == nil do
      :couchbeam_doc.set_value("location", location, doc)
    else
      doc
    end
    Logger.info("new tags: #{tags}")
    doc = case TanukiBackend.get_field_value("tags", doc) do
      nil -> :couchbeam_doc.set_value("tags", tags, doc)
      old_tags ->
        Logger.info("original tags: #{old_tags}")
        merged_tags = :lists.umerge(Enum.sort(old_tags), Enum.sort(tags))
        Logger.info("merged tags: #{merged_tags}")
        :couchbeam_doc.set_value("tags", merged_tags, doc)
    end
    {:ok, doc} = :couchbeam.save_doc(db, doc)
    {id, rev} = :couchbeam_doc.get_idrev(doc)
    Logger.info("document id=#{id} new revision=#{rev}")
  end

  @doc """

  Determine if the given checksum already exists in the database.
  Returns the document id (binary), or nil if not found.

  """
  def find_document(db, checksum) do
    # check if view is defined and return immediately if not
    case :couchbeam.doc_exists(db, "_design/assets") do
      true ->
        options = [{:key, checksum}]
        # Fail fast if unable to check for duplicates, rather than blindly
        # creating more duplicate records.
        {:ok, rows} = :couchbeam_view.fetch(db, {"assets", "by_checksum"}, options)
        # Likewise fail fast if there are multiple records with matching
        # checksums, as that indicates an existing problem that needs
        # immediate attention.
        case length(rows) do
          1 -> :couchbeam_doc.get_value("id", hd(rows))
          0 -> nil
        end
      false -> nil
    end
  end

  @doc """

  Move the named asset to its sharded location. If the asset is an image
  whose orientation needs correction, it will be automatically oriented.
  This changes the content such that its checksum will no longer match.
  However, this ensures that the incoming assets are deduplicated using the
  original contents, not the modified contents which could change in a non-
  deterministic fashion.

  """
  def store_asset(filepath, checksum) do
    # If an existing asset with the same checksum already exists, the new
    # asset will be removed to prevent further processing in the future.
    dst_path = TanukiBackend.checksum_to_asset_path(checksum)
    if File.exists?(dst_path) do
      Logger.info("ignoring duplicate asset #{filepath} with #{checksum}")
      File.rm!(filepath)
    else
      File.mkdir_p!(Path.dirname(dst_path))
      if correct_orientation?(filepath) do
        Logger.info("moving #{filepath} to #{dst_path}")
        # use copy to handle crossing file systems
        {:ok, _bytes_copied} = File.copy(filepath, dst_path)
        File.rm!(filepath)
      else
        auto_orient(filepath, dst_path)
        Logger.info("corrected orientation for #{filepath}, saved to #{dst_path}")
        File.rm!(filepath)
      end
    end
  end

  # Use ImageMagick convert command to read the source file and correct its
  # orientation, writing to the given destination path.
  defp auto_orient(source, destination) do
    cmd = Enum.join(["convert", source, "-auto-orient", destination], " ")
    port = Port.open({:spawn, cmd}, [:exit_status])
    {:ok, 0} = TanukiBackend.wait_for_port(port)
  end

  @doc """

  Remove any superfluous files from the given path, such as those created
  by MacOS (e.g. ".localized", ".DS_Store").

  """
  def delete_extraneous_files(path) do
    extras = [".DS_Store", ".localized"]
    delete_fn = fn(name) ->
      if Enum.member?(extras, name) do
        filepath = Path.join(path, name)
        case File.rm(filepath) do
          :ok -> :ok
          {:error, Reason} ->
            Logger.error("failed to delete file #{filepath}, #{Reason}")
        end
      end
    end
    {:ok, filenames} = File.ls(path)
    Enum.each(filenames, delete_fn)
  end

  @doc """
  Parse a simple date/time string into a date/time tuple. Specifically
  this is suitable for handling the EXIF date/time values. For instance,
  "2014:10:11 13:28:00" would become {{2014,10,11},{13,28,0}}.
  """
  def date_parse(bin) when is_binary(bin) do
    parse_year(to_charlist(bin), [])
  end

  def date_parse(str) when is_list(str) do
    parse_year(str, [])
  end

  defp parse_year([y1, y2, y3, y4 | rest], acc) do
    acc([y1, y2, y3, y4], rest, :year, acc, &parse_month/2)
  end

  defp parse_year(_, _) do
    raise ArgumentError, message: "invalid year (must be 4 digits)"
  end

  defp parse_month([], acc) do
    datetime(acc)
  end

  defp parse_month([?:, m1, m2 | rest], acc) do
    acc([m1, m2], rest, :month, acc, &parse_day/2)
  end

  defp parse_month(_, _) do
    raise ArgumentError, message: "missing : before month"
  end

  defp parse_day([], acc) do
    datetime(acc)
  end

  defp parse_day([?:, d1, d2 | rest], acc) do
    acc([d1, d2], rest, :day, acc, &parse_hour/2)
  end

  defp parse_day(_, _) do
    raise ArgumentError, message: "missing : before day"
  end

  defp parse_hour([], acc) do
    datetime(acc)
  end

  defp parse_hour([?\ , h1, h2 | rest], acc) do
    acc([h1, h2], rest, :hour, acc, &parse_minute/2)
  end

  defp parse_hour(_, _) do
    raise ArgumentError, message: "missing ' ' before day"
  end

  defp parse_minute([], acc) do
    datetime(acc)
  end

  defp parse_minute([?:, m1, m2 | rest], acc) do
    acc([m1, m2], rest, :minute, acc, &parse_second/2)
  end

  defp parse_minute(_, _) do
    raise ArgumentError, message: "missing : before minute"
  end

  defp parse_second([], acc) do
    datetime(acc)
  end

  defp parse_second([?:, s1, s2 | rest], acc) do
    acc([s1, s2], rest, :second, acc, &datetime/2)
  end

  defp parse_second(_, _) do
    raise ArgumentError, message: "missing : before second"
  end

  defp acc(intstr, rest, key, acc, nextf) do
    acc1 = [{key, List.to_integer(intstr)} | acc]
    nextf.(rest, acc1)
  end

  defp datetime(plist) do
    {{
      Keyword.get(plist, :year, 0),
      Keyword.get(plist, :month, 0),
      Keyword.get(plist, :day, 0)
    },
    {
      Keyword.get(plist, :hour, 0),
      Keyword.get(plist, :minute, 0),
      Keyword.get(plist, :second, 0)
    }}
  end

  defp datetime(_, plist) do
    datetime(plist)
  end

  @doc """

  Convert the common Erlang date/time tuple into an array of integers,
  ignoring the seconds (since tanuki does not bother with seconds).

  """
  def time_tuple_to_list({{y, mo, d}, {h, mi, _s}}) do
    [y, mo, d, h, mi]
  end
end
