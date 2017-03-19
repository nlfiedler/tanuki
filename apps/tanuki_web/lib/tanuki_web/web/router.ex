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

  # This goes last because of the glob match at the end.
  scope "/", TanukiWeb.Web do
    pipe_through :browser # Use the default browser stack

    # These two methods are still needed until the Elm http module supports
    # blob/file parts for POST requests. As of 0.18 it only supports string
    # parts for multipart bodies.
    get "/upload", PageController, :upload
    post "/import", PageController, :import

    # These three could go in ApiController, probably, but historically
    # they have been in the PageController.
    get "/asset/:id", PageController, :asset
    get "/thumbnail/:id", PageController, :thumbnail
    get "/preview/:id", PageController, :preview

    # This glob match goes last to catch everything else and direct it to
    # the Elm application.
    get "/*path", PageController, :index
  end
end
