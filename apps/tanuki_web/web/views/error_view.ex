defmodule TanukiWeb.ErrorView do
  use TanukiWeb.Web, :view

  def render("404.html", _assigns) do
    "Page not found"
  end

  def render("500.html", assigns) do
    # Try to get the error details, if possible; sometimes the reason is
    # nil, and sometimes it is an atom, so no message is available.
    try do
      reason = assigns[:reason]
      "Internal server error: #{reason.message}"
    rescue
      _e -> "Internal server error"
    end
  end

  # In case no render clause matches or no
  # template is found, let's render it as 500
  def template_not_found(_template, assigns) do
    render "500.html", assigns
  end
end
