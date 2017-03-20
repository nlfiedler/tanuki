defmodule TanukiWeb.Web.Router do
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

  scope "/", TanukiWeb.Web do
    pipe_through :browser # Use the default browser stack

    get "/", PageController, :index
    get "/year/:id", PageController, :year
    get "/location/:id", PageController, :location
    get "/detail/:id", PageController, :detail
    get "/asset/:id", PageController, :asset
    get "/asset/:id/edit", PageController, :edit
    post "/asset/:id", PageController, :update
    get "/thumbnail/:id", PageController, :thumbnail
    get "/preview/:id", PageController, :preview
    get "/upload", PageController, :upload
    post "/import", PageController, :import
  end

  scope "/api", TanukiWeb.Web do
    pipe_through :api

    get "/tags", ApiController, :tags
    get "/locations", ApiController, :locations
    get "/years", ApiController, :years
    resources "/assets", AssetController, except: [:edit, :new, :delete]
  end

  scope "/admin", TanukiWeb.Web do
    pipe_through :browser # Use the default browser stack

    get "/", AdminController, :index
    post "/rename_tag", AdminController, :rename_tag
    post "/rename_location", AdminController, :rename_location
    post "/tag_to_location", AdminController, :tag_to_location
    post "/sort_tags", AdminController, :sort_tags
    post "/creation_time", AdminController, :creation_time
    post "/original_date", AdminController, :original_date
    post "/incoming", AdminController, :incoming
  end
end
