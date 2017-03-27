defmodule TanukiWeb.Web.ApiControllerTest do
  @moduledoc """

  Because the web tests are akin to integration tests, and they rely on the
  backend being up and running, we keep all of the web app tests in this
  one test suite. Otherwise the tests running concurrently will interfere
  with each other.

  """
  use TanukiWeb.Web.ConnCase
  import ExUnit.CaptureLog

  setup_all do
    # create a temporary workspace for the tests
    priv_dir = Temp.mkdir!("tanuki")
    on_exit(:tmp_dir_rm, fn() -> File.rm_rf(priv_dir) end)
    assets_dir = Path.join(priv_dir, "assets")
    :ok = Application.put_env(:tanuki_backend, :assets_dir, assets_dir)
    thumbnails_dir = Path.join(priv_dir, "thumbnails")
    :ok = Application.put_env(:tanuki_backend, :thumbnails_dir, thumbnails_dir)
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

  test "test bounded_int_value/4" do
    assert TanukiWeb.Web.AssetController.bounded_int_value(8, 5, 5, 10) == 8
    assert TanukiWeb.Web.AssetController.bounded_int_value("8", 5, 5, 10) == 8
    assert TanukiWeb.Web.AssetController.bounded_int_value(nil, 5, 5, 10) == 5
    assert TanukiWeb.Web.AssetController.bounded_int_value("12c", 5, 5, 10) == 5
    assert TanukiWeb.Web.AssetController.bounded_int_value("12.3", 5, 5, 10) == 5
    assert TanukiWeb.Web.AssetController.bounded_int_value("abc", 5, 5, 10) == 5
    assert TanukiWeb.Web.AssetController.bounded_int_value(12, 5, 5, 10) == 10
    assert TanukiWeb.Web.AssetController.bounded_int_value(1, 5, 5, 10) == 5
  end

  test "GET /api/tags", %{conn: conn} do
    conn = get conn, "/api/tags"
    tags = json_response(conn, 200)
    # Because the order in which the tests will run is unknown, we cannot
    # rely on the exact set of results that will come back.
    assert %{"count" => 1, "tag" => "cheeseburger"} in tags
    assert %{"count" => 1, "tag" => "dog"} in tags
    assert %{"count" => 3, "tag" => "joseph"} in tags
  end

  test "GET /api/locations", %{conn: conn} do
    conn = get conn, "/api/locations"
    locations = json_response(conn, 200)
    # Because the order in which the tests will run is unknown, we cannot
    # rely on the exact set of results that will come back.
    assert %{"count" => 1, "location" => "carmel"} in locations
    assert %{"count" => 2, "location" => "santa cruz"} in locations
    assert %{"count" => 2, "location" => "san francisco"} in locations
  end

  test "GET /api/years", %{conn: conn} do
    conn = get conn, "/api/years"
    years = json_response(conn, 200)
    # Because the order in which the tests will run is unknown, we cannot
    # rely on the exact set of results that will come back.
    assert %{"count" => 3, "year" => 2013} in years
    assert %{"count" => 3, "year" => 2014} in years
    assert %{"count" => 4, "year" => 2015} in years
  end

  test "GET /api/assets no tags", %{conn: conn} do
    conn = get conn, "/api/assets"
    body = json_response(conn, 200)
    # some tests add documents, so we cannot assert a specific value
    assert body["count"] > 10
  end

  test "GET /api/assets tag non-list", %{conn: conn} do
    conn = get conn, "/api/assets", [tags: "nomatch"]
    body = json_response(conn, 400)
    assert body["error"] =~ "tags must be list"
  end

  test "GET /api/assets no such tag", %{conn: conn} do
    conn = get conn, "/api/assets", [tags: ["nomatch"]]
    body = json_response(conn, 200)
    assert body["count"] == 0
    assert body["assets"] == []
  end

  test "GET /api/assets matching tag", %{conn: conn} do
    conn = get conn, "/api/assets", [tags: ["picnic"]]
    body = json_response(conn, 200)
    assert length(body["assets"]) == 2
    assert body["count"] == 2
    assert Enum.any?(body["assets"], fn m -> Map.get(m, "filename") == "img0315.jpg" end)
  end

  test "GET /api/assets paging", %{conn: conn} do
    #
    # request all of the matching assets
    #
    conn = get conn, "/api/assets", [tags: ["kittens"]]
    body = json_response(conn, 200)
    assert length(body["assets"]) == 7
    assert body["count"] == 7
    found_ids = for a <- body["assets"], do: Map.get(a, "id")
    assert found_ids == [
      "fighting_kittens", "test_BF", "test_BE", "test_BD", "test_BB", "test_BA", "test_BC"
    ]
    #
    # request the first page of values
    #
    conn = get conn, "/api/assets", [tags: ["kittens"], page: 1, page_size: 2]
    body = json_response(conn, 200)
    assert length(body["assets"]) == 2
    assert body["count"] == 7
    found_ids = for a <- body["assets"], do: Map.get(a, "id")
    assert found_ids == [
      "fighting_kittens", "test_BF"
    ]
    #
    # same page, but string value for page number, and no page_size
    #
    conn = get conn, "/api/assets", [tags: ["kittens"], page: "1"]
    body = json_response(conn, 200)
    assert length(body["assets"]) == 7
    assert body["count"] == 7
    found_ids = for a <- body["assets"], do: Map.get(a, "id")
    assert found_ids == [
      "fighting_kittens", "test_BF", "test_BE", "test_BD", "test_BB", "test_BA", "test_BC"
    ]
    #
    # request a different page of values
    #
    conn = get conn, "/api/assets", [tags: ["kittens"], page: 3, page_size: 2]
    body = json_response(conn, 200)
    assert length(body["assets"]) == 2
    assert body["count"] == 7
    found_ids = for a <- body["assets"], do: Map.get(a, "id")
    assert found_ids == [
      "test_BB", "test_BA"
    ]
    #
    # request the last page of values
    #
    conn = get conn, "/api/assets", [tags: ["kittens"], page: 4, page_size: 2]
    body = json_response(conn, 200)
    assert length(body["assets"]) == 1
    assert body["count"] == 7
    found_ids = for a <- body["assets"], do: Map.get(a, "id")
    assert found_ids == [
      "test_BC"
    ]
  end

  test "GET /api/assets/:id missing asset", %{conn: conn} do
    conn = get conn, "/api/assets/nosuch"
    body = json_response(conn, 404)
    assert body["error"] =~ "no such asset"
  end

  test "GET /api/assets/:id existing asset", %{conn: conn} do
    conn = get conn, "/api/assets/test_AC"
    body = json_response(conn, 200)
    assert Map.get(body, "filename") == "img0315.jpg"
  end

  test "POST /api/assets create new asset", %{conn: conn} do
    # make a copy of the input file as it will be (re)moved
    File.copy!("./test/fixtures/ash_tree.jpg", "./test/fixtures/example.jpg")
    upload = %Plug.Upload{
      path: "./test/fixtures/example.jpg",
      content_type: "image/jpeg",
      filename: "example.jpg"
    }
    assert capture_log(fn ->
      conn = post conn, "/api/assets", %{:asset => upload}
      body = json_response(conn, 200)
      assert Map.get(body, "status") == "success"
    end) =~ "unable to read EXIF data"
  end

  test "PUT /api/assets/test_AC update asset", %{conn: conn} do
    # retrieve existing asset, test known field value
    conn = get conn, "/api/assets/test_AC"
    body = json_response(conn, 200)
    assert Map.get(body, "filename") == "img0315.jpg"
    # update the document
    params = %{
      "location" => "outside",
      "caption" => "how interesting",
      # must have 'picnic' so other tests keep working
      "tags" => "picnic,outside,grass",
      "user_date" => ""
    }
    conn = put conn, "/api/assets/test_AC", params
    body = json_response(conn, 200)
    assert Map.get(body, "status") == "success"
    # retrieve the updated document and test updated field
    conn = get conn, "/api/assets/test_AC"
    body = json_response(conn, 200)
    assert Map.get(body, "caption") == "how interesting"
    assert Map.get(body, "tags") == ["grass", "outside", "picnic"]
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
end
