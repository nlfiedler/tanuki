defmodule TanukiIncomingTest do
  use ExUnit.Case
  import ExUnit.CaptureLog

  setup_all do
    # create a temporary workspace for the tests
    priv_dir = Temp.mkdir!("tanuki")
    on_exit(:tmp_dir_rm, fn() -> File.rm_rf(priv_dir) end)
    assets_dir = Path.join(priv_dir, "assets")
    :ok = Application.load(:tanuki_backend)
    :ok = Application.put_env(:tanuki_backend, :assets_dir, assets_dir)
    incoming_dir = Path.join(priv_dir, "incoming")
    :ok = Application.load(:tanuki_incoming)
    :ok = Application.put_env(:tanuki_incoming, :incoming_dir, incoming_dir)
    url = Application.get_env(:tanuki_incoming, :couchdb_url)
    opts = Application.get_env(:tanuki_incoming, :couchdb_opts)
    server = :couchbeam.server_connection(url, opts)
    # clean up from a previous test run
    database = Application.get_env(:tanuki_incoming, :database)
    if :couchbeam.db_exists(server, database) do
      {:ok, _wat} = :couchbeam.delete_db(server, database)
    end
    {:ok, db} = :couchbeam.create_db(server, database)
    # now that everything is properly configured, start the applications
    {:ok, _started} = Application.ensure_all_started(:tanuki_backend)
    {:ok, _started} = Application.ensure_all_started(:tanuki_incoming)
    {:ok, db: db, incoming_dir: incoming_dir}
  end

  test "is file a jpeg?" do
    assert TanukiIncoming.jpeg?("./test/fixtures/img_015.JPG")
    refute TanukiIncoming.jpeg?("./test/fixtures/LICENSE.txt")
  end

  test "parsing date/time strings" do
    assert TanukiIncoming.date_parse("2014:10:11 13:28:16") == {:ok, {{2014, 10, 11}, {13, 28, 16}}}
    assert TanukiIncoming.date_parse("2014:10:11 13:28") == {:ok, {{2014, 10, 11}, {13, 28, 0}}}
    assert TanukiIncoming.date_parse("2014:10:11 13") == {:ok, {{2014, 10, 11}, {13, 0, 0}}}
    assert TanukiIncoming.date_parse("2014:10:11") == {:ok, {{2014, 10, 11}, {0, 0, 0}}}
    assert TanukiIncoming.date_parse("2014:10") == {:ok, {{2014, 10, 0}, {0, 0, 0}}}
    assert TanukiIncoming.date_parse("2014") == {:ok, {{2014, 0, 0}, {0, 0, 0}}}
    assert TanukiIncoming.date_parse("2014-10-11 13:28:00") == {:ok, {{2014, 10, 11}, {13, 28, 0}}}
    assert TanukiIncoming.date_parse("2014-10-11T13:28:00Z") == {:ok, {{2014, 10, 11}, {13, 28, 0}}}
    assert TanukiIncoming.date_parse("2014-10-11T13:28:14.000Z") == {:ok, {{2014, 10, 11}, {13, 28, 14}}}
    assert TanukiIncoming.date_parse("2014/10/11 01:28") == :error
    assert TanukiIncoming.date_parse("not a date") == :error
  end

  test "reading exif date from file" do
    # images with Exif data and an original date
    assert TanukiIncoming.get_original_date("./test/fixtures/img_015.JPG") == [2011, 10, 7, 16, 18]
    assert TanukiIncoming.get_original_date("./test/fixtures/IMG_5745.JPG") == [2014, 4, 23, 13, 33]
    assert TanukiIncoming.get_original_date("./test/fixtures/dcp_1069.jpg") == [2003, 9, 3, 17, 24]
    # an image that lacks the original date field
    level = Logger.level()
    Logger.configure(level: :info)
    assert capture_log(fn ->
      assert TanukiIncoming.get_original_date("./test/fixtures/fighting_kittens.jpg") == :null
    end) =~ "unable to read EXIF data"
    # something that is not a jpeg file
    assert capture_log(fn ->
      assert TanukiIncoming.get_original_date("./test/fixtures/LICENSE.txt") == :null
    end) =~ "unable to read creation_time"
    Logger.configure(level: level)
  end

  test "compute SHA256 checksum" do
    # a file larger than our read buffer
    {:ok, checksum} = TanukiIncoming.compute_checksum("./test/fixtures/img_015.JPG")
    assert checksum == "d09fd659423e71bb1b5e20d78a1ab7ce393e74e463f2dface3634d78ec155397"
    # a file much smaller than our read buffer
    {:ok, checksum} = TanukiIncoming.compute_checksum("./test/fixtures/LICENSE.txt")
    assert checksum == "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
  end

  test "folder name to fields conversion" do
    assert TanukiIncoming.convert_path_to_details("") == {[], nil}
    assert TanukiIncoming.convert_path_to_details("foo") == {["foo"], nil}
    assert TanukiIncoming.convert_path_to_details("foo__bar") == {["foo", "bar"], nil}
    assert TanukiIncoming.convert_path_to_details("foo@home") == {["foo"], "home"}
    assert TanukiIncoming.convert_path_to_details("foo_bar_baz@out_on_the_town") ==
      {["foo", "bar", "baz"], "out on the town"}
    assert TanukiIncoming.convert_path_to_details("@home") == {[], "home"}
  end

  test "image orientation correction" do
    # correctly oriented ("Top-left")
    assert TanukiIncoming.correct_orientation?("./test/fixtures/IMG_5745.JPG")
    # not correctly oriented ("Bottom-right")
    refute TanukiIncoming.correct_orientation?("./test/fixtures/IMG_8841.JPG")
    # not an image so "correctly" oriented
    assert TanukiIncoming.correct_orientation?("./test/fixtures/LICENSE.txt")
    # missing EXIF data, but ImageMagick still detects rotation
    refute TanukiIncoming.correct_orientation?("./test/fixtures/fighting_kittens.jpg")
  end

  test "create new document", context do
    db = context[:db]
    fullpath = "./test/fixtures/IMG_5745.JPG"
    tags = ["foo", "bar"]
    location = "outside"
    sha = "cafebabe"
    {:ok, id} = TanukiIncoming.create_document(db, fullpath, tags, location, sha)
    {:ok, doc} = :couchbeam.open_doc(db, id)
    assert :couchbeam_doc.get_value("file_name", doc) == "IMG_5745.JPG"
    assert :couchbeam_doc.get_value("file_size", doc) == 107302
    assert :couchbeam_doc.get_value("location", doc) == location
    # the tags will get sorted on the way in
    assert :couchbeam_doc.get_value("tags", doc) == ["bar", "foo"]
  end

  test "update existing document", context do
    db = context[:db]
    fullpath = "./test/fixtures/IMG_5745.JPG"
    tags = ["foo", "bar"]
    location = "outside"
    sha = "cafebabe007"
    {:ok, id} = TanukiIncoming.create_document(db, fullpath, tags, location, sha)
    {:ok, doc} = :couchbeam.open_doc(db, id)
    assert :couchbeam_doc.get_value("file_name", doc) == "IMG_5745.JPG"
    assert :couchbeam_doc.get_value("file_size", doc) == 107302
    assert :couchbeam_doc.get_value("location", doc) == location
    # the tags will get sorted on the way in
    assert :couchbeam_doc.get_value("tags", doc) == ["bar", "foo"]

    tags = ["foo", "bar", "quux", "baz"]
    TanukiIncoming.update_document(db, id, tags, "home")
    {:ok, doc} = :couchbeam.open_doc(db, id)
    # location was already set, so it does not get set again
    assert :couchbeam_doc.get_value("location", doc) == location
    # the tags will get sorted and merged on the way in
    assert :couchbeam_doc.get_value("tags", doc) == ["bar", "baz", "foo", "quux"]
  end

  test "find existing document by checksum", context do
    db = context[:db]
    fullpath = "./test/fixtures/IMG_5745.JPG"
    tags = ["foo", "bar"]
    location = "outside"
    sha = "cafebabea113"
    {:ok, id} = TanukiIncoming.create_document(db, fullpath, tags, location, sha)
    {:ok, _doc} = :couchbeam.open_doc(db, id)
    assert TanukiIncoming.find_document(db, sha) == id
    assert TanukiIncoming.find_document(db, "doesnotexist") == nil
  end

  test "deleting extraneous files" do
    File.write!("./test/fixtures/.DS_Store", "dummy")
    File.write!("./test/fixtures/.localized", "dummy")
    TanukiIncoming.delete_extraneous_files("./test/fixtures")
    refute File.exists?("./test/fixtures/.DS_Store")
    refute File.exists?("./test/fixtures/.localized")
  end

  test "storing an asset", context do
    incoming_dir = context[:incoming_dir]
    File.mkdir_p!(incoming_dir)
    test_asset_fn = fn(name, sha256) ->
      fix_path = Path.join("./test/fixtures", name)
      src_path = Path.join(incoming_dir, name)
      File.copy!(fix_path, src_path)
      # not blowing up basically means it is working
      TanukiIncoming.store_asset(src_path, sha256)
      dst_path = TanukiBackend.checksum_to_asset_path(sha256)
      assert File.exists?(dst_path)
    end
    # 1. file that does not need orientation correction
    # 2. file that is not an image
    # 3. file that needs orientation correction
    # 4. one of the above files again
    names = [
      "img_015.JPG",
      "LICENSE.txt",
      "IMG_8841.JPG",
      "IMG_8841.JPG"
    ]
    checksums = [
      "d09fd659423e71bb1b5e20d78a1ab7ce393e74e463f2dface3634d78ec155397",
      "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30",
      "a1f33723e44f23168edee008a148846accc04f20274037742b17956e40fc7fde",
      "a1f33723e44f23168edee008a148846accc04f20274037742b17956e40fc7fde"
    ]
    expected = Enum.zip(names, checksums)
    for {name, checksum} <- expected, do: test_asset_fn.(name, checksum)
  end

  test "importing assets", context do
    incoming_dir = context[:incoming_dir]
    tagged_dir = Path.join(incoming_dir, "testing^various_sundry_assets@nowhere")
    File.mkdir_p!(tagged_dir)
    prepare_asset_fn = fn(filename) ->
      fix_path = Path.join("./test/fixtures", filename)
      src_path = Path.join(tagged_dir, filename)
      File.copy!(fix_path, src_path)
    end
    # Test everything, from non-images to images needing orientation
    # correction, and images that lack EXIF date information.
    input_files = [
      "IMG_5745.JPG",
      "LICENSE.txt",
      "dcp_1069.jpg",
      "fighting_kittens.jpg",
      "img_015.JPG"
    ]
    for file <- input_files, do: prepare_asset_fn.(file)
    # At this point we have tested every single function individually, with
    # all the usual cases covered, so the fact that this does not blow up
    # and apparently made a bunch of assets go somewhere means it almost
    # certainly worked.
    level = Logger.level()
    Logger.configure(level: :info)
    assert capture_log(fn ->
      {:ok, 5} = GenServer.call(TanukiIncoming, :process_now)
    end) =~ "unable to read EXIF data"
    Logger.configure(level: level)
    assert File.ls!(incoming_dir) == []
    rows = TanukiBackend.by_tags(["sundry"])
    assert length(rows) == 5
  end
end
