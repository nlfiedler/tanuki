defmodule TanukiWeb.Web.ApiControllerTest do
  use TanukiWeb.Web.ConnCase

  test "GET /api/tags", %{conn: conn} do
    conn = get conn, "/api/tags"
    tags = json_response(conn, 200)
    # [%{"count" => 5, "tag" => "kittens"}, ...]
    assert Enum.any?(tags, fn m -> Map.has_key?(m, "tag") end)
  end
end
