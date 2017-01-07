defmodule TanukiWeb.Router do
  use TanukiWeb.Web, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_flash
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", TanukiWeb do
    pipe_through :browser # Use the default browser stack

    get "/", PageController, :index
    get "/detail/:id", PageController, :detail
    get "/asset/:id", PageController, :asset
    get "/asset/:id/edit", PageController, :edit
    post "/asset/:id", PageController, :update
    get "/thumbnail/:id", PageController, :thumbnail
  end

  # Other scopes may use custom stacks.
  # scope "/api", TanukiWeb do
  #   pipe_through :api
  # end
end
