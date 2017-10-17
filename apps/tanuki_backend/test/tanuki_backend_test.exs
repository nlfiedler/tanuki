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
    {:ok, db} = :couchbeam.create_db(server, database)
    add_test_docs(db)
    # set up mnesia
    :ok = Application.stop(:mnesia)
    :ok = Application.put_env(:mnesia, :dir, priv_dir)
    :ok = Application.start(:mnesia)
    TanukiBackend.ensure_schema([:erlang.node()])
    # now that everything is properly configured, start the application
    {:ok, _started2} = Application.ensure_all_started(:tanuki_backend)
    {:ok, db: db}
  end

  test "fetch documents" do
    assert TanukiBackend.fetch_document("does_not_exist") == {:error, :not_found}
    {:ok, doc} = TanukiBackend.fetch_document("test_AD")
    filename = :couchbeam_doc.get_value("file_name", doc)
    assert filename == "img0315.jpg"
  end

  test "count assets" do
    assert TanukiBackend.count_assets() == 8
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
    assert length(rows) == 7
    validate_fn = fn(row, {key, value}) ->
      assert :couchbeam_doc.get_value("key", row) == key
      assert :couchbeam_doc.get_value("value", row) == value
    end
    key_values = [
      {"cat", 2},
      {"cheeseburger", 1},
      {"christina", 3},
      {"dog", 1},
      {"joseph", 3},
      {"kittens", 1},
      {"picnic", 2}]
    expected = Enum.zip(rows, key_values)
    for {row, key_value} <- expected, do: validate_fn.(row, key_value)
  end

  test "fetching list of all years" do
    rows = TanukiBackend.all_years()
    assert length(rows) == 4
    validate_fn = fn(row, {key, value}) ->
      assert :couchbeam_doc.get_value("key", row) == key
      assert :couchbeam_doc.get_value("value", row) == value
    end
    key_values = [{2013, 1}, {2014, 2}, {2015, 2}, {2017, 2}]
    expected = Enum.zip(rows, key_values)
    for {row, key_value} <- expected, do: validate_fn.(row, key_value)
  end

  test "fetching list of all locations" do
    rows = TanukiBackend.all_locations()
    assert length(rows) == 4
    validate_fn = fn(row, {key, value}) ->
      assert :couchbeam_doc.get_value("key", row) == key
      assert :couchbeam_doc.get_value("value", row) == value
    end
    key_values = [{"carmel", 1}, {"outside", 1}, {"san francisco", 2}, {"santa cruz", 2}]
    expected = Enum.zip(rows, key_values)
    for {row, key_value} <- expected, do: validate_fn.(row, key_value)
  end

  test "fetching metadata by checksum" do
    checksum = "39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100"
    rows = TanukiBackend.by_checksum(checksum)
    assert length(rows) == 1
    validate_fn = fn(row, {id, value}) ->
      assert :couchbeam_doc.get_value("id", row) == id
      assert :couchbeam_doc.get_value("value", row) == value
    end
    id_values = [{"test_AA", "image/jpeg"}]
    for {row, id_value} <- Enum.zip(rows, id_values), do: validate_fn.(row, id_value)
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

    # use the identical api called query/3
    rows = TanukiBackend.query(["christina", "joseph"], nil, nil)
    for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)

    # negative case, no such tags
    assert TanukiBackend.by_tags(["foo", "bar"]) == []
  end

  test "retrieving results by tags and years" do
    rows = TanukiBackend.query(["christina", "joseph"], [2015], nil)
    assert length(rows) == 2
    validate_fn = fn(row, id) ->
      assert :couchbeam_doc.get_value("id", row) == id
    end
    ids = ["test_AD", "test_AE"]
    for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)

    # negative case, no such tags
    assert TanukiBackend.query(["foo", "bar"], [2015], nil) == []
  end

  test "retrieving results by tags and locations" do
    rows = TanukiBackend.query(["christina", "joseph"], nil, ["carmel", "santa cruz"])
    assert length(rows) == 3
    validate_fn = fn(row, id) ->
      assert :couchbeam_doc.get_value("id", row) == id
    end
    ids = ["test_AD", "test_AE", "test_AF"]
    for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)

    # negative case, no such tags
    assert TanukiBackend.query(["foo", "bar"], nil, ["nowhere"]) == []
  end

  test "retrieving results by tags, years, and locations" do
    rows = TanukiBackend.query(["christina", "joseph"], [2015], ["carmel"])
    assert length(rows) == 1
    validate_fn = fn(row, id) ->
      assert :couchbeam_doc.get_value("id", row) == id
    end
    ids = ["test_AD"]
    for {row, id} <- Enum.zip(rows, ids), do: validate_fn.(row, id)

    # negative case, no such tags
    assert TanukiBackend.query(["foo", "bar"], [2015], ["nowhere"]) == []
  end

  test "retrieving by year" do
    validate_fn = fn({input, count, expected_ids}) ->
      rows = TanukiBackend.by_date(input)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: assert :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [
      {2013, 1, ["test_AA"]},
      {2014, 2, ["test_AC", "test_AB"]},
      {2015, 2, ["test_AD", "test_AE", "test_AF"]}
    ]
    for input <- inputs, do: validate_fn.(input)

    # negative case, no such year
    assert TanukiBackend.by_date(1973) == []
  end

  test "retrieving by multiple years" do
    validate_fn = fn({input, count, expected_ids}) ->
      rows = TanukiBackend.query(nil, input, nil)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: assert :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [
      {[2013, 2014], 3, ["test_AA", "test_AC", "test_AB"]},
      {[2015], 2, ["test_AD", "test_AE"]}
    ]
    for input <- inputs, do: validate_fn.(input)

    # negative case, no such year
    assert TanukiBackend.query(nil, [1973], nil) == []
  end

  test "retrieving by years and locations" do
    validate_fn = fn({years, locations, count, expected_ids}) ->
      rows = TanukiBackend.query(nil, years, locations)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: assert :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [
      {[2013, 2014], ["san francisco"], 2, ["test_AA", "test_AB"]}
    ]
    for input <- inputs, do: validate_fn.(input)

    # negative case, no such year, location
    assert TanukiBackend.query(nil, [1973], ["nowhere"]) == []
  end

  test "retrieving by location" do
    validate_fn = fn({input, count, expected_ids}) ->
      rows = TanukiBackend.by_location(input)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: assert :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [
      {"carmel", 1, ["test_AD"]},
      {"san francisco", 2, ["test_AA", "test_AB"]},
      {"santa cruz", 2, ["test_AE", "test_AF"]}
    ]
    for input <- inputs, do: validate_fn.(input)

    # negative case, no such location
    assert TanukiBackend.by_location("philadelphia") == []
  end

  test "retrieving by multiple locations" do
    validate_fn = fn({input, count, expected_ids}) ->
      rows = TanukiBackend.query(nil, nil, input)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: assert :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [
      {["carmel", "san francisco"], 3, ["test_AD", "test_AA", "test_AB"]},
      {["santa cruz"], 2, ["test_AE", "test_AF"]}
    ]
    for input <- inputs, do: validate_fn.(input)

    # negative case, no such location
    assert TanukiBackend.query(nil, nil, ["philadelphia"]) == []
  end

  test "retrieving by year and month" do
    validate_fn = fn({{year, month}, count, expected_ids}) ->
      rows = TanukiBackend.by_date(year, month)
      assert length(rows) == count
      for {id, row} <- Enum.zip(expected_ids, rows),
        do: :couchbeam_doc.get_value("id", row) == id
    end
    inputs = [
      {{2014, 7}, 1, ["test_AC"]},
      {{2014, 10}, 1, ["test_AB"]},
      {{2015, 4}, 2, ["test_AD", "test_AE"]}
    ]
    for input <- inputs, do: validate_fn.(input)
  end

  test "date/time formatting" do
    assert TanukiBackend.date_list_to_string([2014, 12, 23, 22, 28]) == "2014-12-23 22:28"
    assert TanukiBackend.date_list_to_string([2016, 12, 23, 8, 5]) == "2016-12-23 08:05"
    assert TanukiBackend.date_list_to_string([33, 5, 2, 8, 5]) == "0033-05-02 08:05"
    assert TanukiBackend.date_list_to_string([2017, 1, 14, 1, 1], :date_only) == "2017-01-14"
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
      {:ok, binary, _mimetype} = TanukiBackend.generate_thumbnail(checksum, :thumbnail)
      assert is_binary(binary)
      assert byte_size(binary) < 20000
    end) =~ "asset file does not exist"
  end

  test "generating thumbnail" do
    checksum = install_asset("fighting_kittens.jpg")
    {:ok, binary, mimetype} = TanukiBackend.generate_thumbnail(checksum, :thumbnail)
    assert "image/jpeg" == mimetype
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

  test "converting JavaScript to view map" do
    # without a reduce function
    result = TanukiBackend.Server.read_view_js("./test/fixtures/by_checksum.js")
    expected = {
      "by_checksum", {[
        {"map", "function(doc) { if (doc.sha256) { emit(doc.sha256, doc.mimetype); } }"}
      ]}
    }
    assert result == expected

    # with a reduce function
    result = TanukiBackend.Server.read_view_js("./test/fixtures/tags.js")
    expected = {
      "tags", {[
        {"map", "function (doc) { if (doc.tags && Array.isArray(doc.tags)) { " <>
                 "doc.tags.forEach(function (tag) { emit(tag.toLowerCase(), 1); }); } }"},
        {"reduce", "_count"}
      ]}
    }
    assert result == expected
  end

  test "installing views", context do
    db = context[:db]
    doc_id = "_design/assets"
    # Make sure the designs document exists at rev '1', then delete it
    # which will result in another revision (visible in the history).
    assert :couchbeam.doc_exists(db, doc_id)
    {:ok, doc} = :couchbeam.open_doc(db, doc_id)
    rev = :couchbeam_doc.get_rev(doc)
    assert String.starts_with?(rev, "1-")
    :couchbeam.delete_doc(db, doc)

    # install the designs and ensure the revision is now '3'
    level = Logger.level()
    Logger.configure(level: :info)
    assert capture_log(fn ->
      :ok = TanukiBackend.Server.install_designs(db)
    end) =~ "created _design/assets document"
    {:ok, doc} = :couchbeam.open_doc(db, doc_id)
    rev = :couchbeam_doc.get_rev(doc)
    assert String.starts_with?(rev, "3-")

    # do it again and ensure the revision is still '3'
    assert capture_log(fn ->
      :ok = TanukiBackend.Server.install_designs(db)
    end) == ""
    {:ok, doc} = :couchbeam.open_doc(db, doc_id)
    rev = :couchbeam_doc.get_rev(doc)
    assert String.starts_with?(rev, "3-")
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
