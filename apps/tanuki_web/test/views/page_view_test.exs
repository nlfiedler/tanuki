defmodule TanukiWeb.Web.PageViewTest do
  use TanukiWeb.Web.ConnCase, async: true

  test "quicktime mimetype to mp4" do
    assert TanukiWeb.Web.PageView.mimetype("video/quicktime") == "video/mp4"
  end

  test "other mimetypes returned as-is" do
    assert TanukiWeb.Web.PageView.mimetype("video/ogg") == "video/ogg"
    assert TanukiWeb.Web.PageView.mimetype("image/jpeg") == "image/jpeg"
  end
end
