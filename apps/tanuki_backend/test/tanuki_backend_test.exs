defmodule TanukiBackendTest do
  use ExUnit.Case
  import ExUnit.CaptureLog

  setup_all do
    # create a temporary workspace for the tests
    priv_dir = Temp.mkdir!("tanuki")
    on_exit(:tmp_dir_rm, fn() -> File.rm_rf(priv_dir) end)
    assets_dir = Path.join(priv_dir, "assets")
    :ok = Application.put_env(:tanuki_backend, :assets_dir, assets_dir)
    url = Application.get_env(:tanuki_backend, :couchdb_url)
    opts = Application.get_env(:tanuki_backend, :couchdb_opts)
    server = :couchbeam.server_connection(url, opts)
    # clean up from a previous test run
    database = Application.get_env(:tanuki_backend, :database)
    if :couchbeam.db_exists(server, database) do
      {:ok, _wat} = :couchbeam.delete_db(server, database)
    end
    {:ok, db} = :couchbeam.create_db(server, database, [])
    add_test_docs(db)
    # set up mnesia
    :ok = Application.stop(:mnesia)
    :ok = Application.put_env(:mnesia, :dir, priv_dir)
    :ok = Application.start(:mnesia)
    TanukiBackend.ensure_schema([:erlang.node()])
    # now that everything is properly configured, start the application
    {:ok, _started2} = Application.ensure_all_started(:tanuki_backend)
    :ok
  end

  test "fetch documents" do
    assert TanukiBackend.fetch_document("does_not_exist") == {:error, :not_found}
    {:ok, doc} = TanukiBackend.fetch_document("test_AD")
    filename = :couchbeam_doc.get_value("file_name", doc)
    assert filename == "img0315.jpg"
  end

  test "update document" do
    # this test may run before the one above, so use a different document
    {:ok, doc} = TanukiBackend.fetch_document("test_AC")
    newdoc = :couchbeam_doc.set_value("file_name", "foo.bar", doc)
    {:ok, updated} = TanukiBackend.update_document(newdoc)
    assert :couchbeam_doc.get_value("file_name", updated) == "foo.bar"
  end

  test "fetching list of all tags" do
    rows = TanukiBackend.all_tags()
    assert length(rows) == 6
    validate_fn = fn(row, key, value) ->
      assert :couchbeam_doc.get_value("key", row) == key
      assert :couchbeam_doc.get_value("value", row) == value
    end
    keys = ["cat", "cheeseburger", "christina", "dog", "joseph", "picnic"]
    values = [2, 1, 3, 1, 3, 2]
    expected = Enum.zip([rows, keys, values])
    for {row, key, value} <- expected, do: validate_fn.(row, key, value)
  end

  test "fetching metadata by checksum" do
    checksum = "39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100"
    rows = TanukiBackend.by_checksum(checksum)
    assert length(rows) == 1
    validate_fn = fn(row, id, value) ->
      assert :couchbeam_doc.get_value("id", row) == id
      assert :couchbeam_doc.get_value("value", row) == value
    end
    ids = ["test_AA"]
    values = ["image/jpeg"]
    for {row, id, value} <- Enum.zip([rows, ids, values]), do: validate_fn.(row, id, value)
    # negative case, no matching checksum
    assert TanukiBackend.by_checksum("cafebabe") == []
  end

  test "retrieving results by tags" do
    rows = TanukiBackend.by_tags(["christina", "joseph"])
    assert length(rows) == 3
    validate_fn = fn(row, id) ->
      assert :couchbeam_doc.get_value("id", row) == id
    end
    ids = ["test_AD", "test_AE", "test_AF"]
    for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)

    # request the same set of tags again to ensure caching is not broken
    level = Logger.level()
    Logger.configure(level: :info)
    assert capture_log(fn ->
      rows = TanukiBackend.by_tags(["christina", "joseph"])
      assert length(rows) == 3
      for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)
    end) =~ "cache hit for"

    # Invoke by_tags/2 to test the sorting function support.
    # Passing a (different) sort function causes a cache miss.
    assert capture_log(fn ->
      rows = TanukiBackend.by_tags(["christina", "joseph"], fn(a, b) ->
        # The date is the first value in the list of "value" in the row in the
        # 'by_tag' CouchDB view. By default sort newer assets before older.
        a_date = hd(:couchbeam_doc.get_value("value", a))
        b_date = hd(:couchbeam_doc.get_value("value", b))
        a_date >= b_date
      end)
      assert length(rows) == 3
      for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)
    end) =~ "cache miss for"

    # restore the logger configuration
    Logger.configure(level: level)

    # negative case, no such tags
    assert TanukiBackend.by_tags(["foo", "bar"]) == []
  end

  test "retrieving by year" do
    validate_fn = fn(input, count, expected_ids) ->
      rows = TanukiBackend.by_date(input)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: assert :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [2013, 2014, 2015]
    counts = [1, 2, 3]
    ids = [
      ["test_AA"],
      ["test_AC", "test_AB"],
      ["test_AD", "test_AE", "test_AF"]
    ]
    for {i, c, e} <- Enum.zip([inputs, counts, ids]), do: validate_fn.(i, c, e)
  end

  test "retrieving by year and month" do
    validate_fn = fn({year, month}, count, expected_ids) ->
      rows = TanukiBackend.by_date(year, month)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [{2014, 7}, {2014, 10}, {2015, 4}]
    counts = [1, 1, 3]
    ids = [
      ["test_AC"],
      ["test_AB"],
      ["test_AD", "test_AE", "test_AF"]
    ]
    for {i, c, e} <- Enum.zip([inputs, counts, ids]), do: validate_fn.(i, c, e)
  end

  test "date/time formatting" do
    assert TanukiBackend.date_list_to_string([2014, 12, 23, 22, 28]) == "2014/12/23 22:28"
    assert TanukiBackend.date_list_to_string([2016, 12, 23, 8, 5]) == "2016/12/23 8:05"
    assert TanukiBackend.date_list_to_string([33, 5, 2, 8, 5]) == "33/5/2 8:05"
    assert TanukiBackend.date_list_to_string([2017, 1, 14, 1, 1], :date_only) == "2017/1/14"
  end

  test "checksum to asset path" do
    checksum = "d4c1c8f7543575872ed89179fbe874792a5925f84834c7d61b54bb2c735a5040"
    p = TanukiBackend.checksum_to_asset_path(checksum)
    ad = Application.get_env(:tanuki_backend, :assets_dir)
    assert p == "#{ad}/d4/c1/c8f7543575872ed89179fbe874792a5925f84834c7d61b54bb2c735a5040"
  end

  test "generating thumbnail for missing asset" do
    # A thumbnail for something that does not exist or cannot be read for
    # some reason should still return a binary, as well as log the error.
    checksum = "cafebabe"
    assert capture_log(fn ->
      binary = TanukiBackend.generate_thumbnail(checksum, :thumbnail)
      assert is_binary(binary)
      assert byte_size(binary) < 20000
    end) =~ "unable to stat asset file"
  end

  test "generating thumbnail" do
    checksum = install_asset("fighting_kittens.jpg")
    binary = TanukiBackend.generate_thumbnail(checksum, :thumbnail)
    assert is_binary(binary)
    assert byte_size(binary) < 20000
  end

  test "retrieving cached thumbnail" do
    checksum = install_asset("fighting_kittens.jpg")
    # Reset the logger level to :info so we can capture the logging about
    # the thumbnail cache. Other applications under the umbrella project
    # may configure the level differently, hence this override.
    level = Logger.level()
    Logger.configure(level: :info)
    assert capture_log(fn ->
      {:ok, binary, mimetype} = TanukiBackend.retrieve_thumbnail(checksum)
      assert "image/jpeg" == mimetype
      assert is_binary(binary)
      assert byte_size(binary) < 20000
    end) =~ "cache miss for thumbnail"
    # multiple requests for the same asset should hit the cache
    assert capture_log(fn ->
      {:ok, _binary, _mimetype} = TanukiBackend.retrieve_thumbnail(checksum)
    end) =~ "cache hit for thumbnail"
    Logger.configure(level: level)
  end

  defp add_test_docs(db) do
    # populate test database from json files in fixtures
    insert_doc_fn = fn(filename) ->
      filepath = Path.join("./test/fixtures", filename)
      binary = File.read!(filepath)
      json = :couchbeam_ejson.decode(binary)
      {:ok, _doc1} = :couchbeam.save_doc(db, json)
    end
    json_selector_fn = fn(name) -> Path.extname(name) == ".json" end
    json_files = Enum.filter(File.ls!("./test/fixtures"), json_selector_fn)
    for filename <- json_files, do: insert_doc_fn.(filename)
  end

  # Copy the given file from "test/fixtures" into the test-configured
  # assets directory, if it is not already there.
  defp install_asset(filename) do
    src_path = Path.join("./test/fixtures", filename)
    {:ok, binary} = File.read(src_path)
    sha256 = Base.encode16(:crypto.hash(:sha256, binary), case: :lower)
    dst_path = TanukiBackend.checksum_to_asset_path(sha256)
    unless File.exists?(dst_path) do
      File.mkdir_p!(Path.dirname(dst_path))
      File.copy!(src_path, dst_path)
    end
    sha256
  end
end
