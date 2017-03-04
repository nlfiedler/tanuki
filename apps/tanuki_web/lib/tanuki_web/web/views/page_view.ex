defmodule TanukiWeb.Web.PageView do
  use TanukiWeb.Web, :view

  # Because Chrome refuses to play Quicktime videos as-is and Quicktime
  # basically is MP4 for all practical purposes.
  def mimetype("video/quicktime"), do: "video/mp4"
  def mimetype(other), do: other
end
