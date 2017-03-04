defmodule TanukiWeb.Web.AdminController do
  use TanukiWeb.Web, :controller

  def index(conn, _params) do
    render(conn, :index)
  end

  def rename_tag(conn, params) do
    old_tag = params["old_tag"]
    new_tag = params["new_tag"]

    # establish a connection for the work we will be doing
    url = Application.get_env(:tanuki_backend, :couchdb_url)
    opts = Application.get_env(:tanuki_backend, :couchdb_opts)
    db_name = Application.get_env(:tanuki_backend, :database)
    server = :couchbeam.server_connection(url, opts)
    {:ok, db} = :couchbeam.open_or_create_db(server, db_name)

    # get all the docs with the old tag
    {:ok, rows} = :couchbeam_view.fetch(db, {"assets", "by_tag"}, [{:key, old_tag}])
    Enum.each(rows, fn(row) ->
      doc_id = :couchbeam_doc.get_value("id", row)
      {:ok, doc} = :couchbeam.open_doc(db, doc_id)
      old_tags = :couchbeam_doc.get_value("tags", doc)
      new_tags = Enum.uniq(Enum.sort(List.delete(old_tags, old_tag) ++ [new_tag]))
      new_doc = :couchbeam_doc.set_value("tags", new_tags, doc)
      {:ok, _doc1} = :couchbeam.save_doc(db, new_doc)
    end)

    conn
    |> put_flash(:info, "Updated #{length(rows)} documents")
    |> render(:index)
  end

  def rename_location(conn, params) do
    old_location = params["old_location"]
    new_location = params["new_location"]

    # establish a connection for the work we will be doing
    url = Application.get_env(:tanuki_backend, :couchdb_url)
    opts = Application.get_env(:tanuki_backend, :couchdb_opts)
    db_name = Application.get_env(:tanuki_backend, :database)
    server = :couchbeam.server_connection(url, opts)
    {:ok, db} = :couchbeam.open_or_create_db(server, db_name)

    {:ok, rows} = :couchbeam_view.fetch(db, {"assets", "by_location"}, [{:key, old_location}])
    Enum.each(rows, fn(row) ->
      doc_id = :couchbeam_doc.get_value("id", row)
      {:ok, doc} = :couchbeam.open_doc(db, doc_id)
      new_doc = :couchbeam_doc.set_value("location", new_location, doc)
      {:ok, _doc1} = :couchbeam.save_doc(db, new_doc)
    end)

    conn
    |> put_flash(:info, "Updated #{length(rows)} documents")
    |> render(:index)
  end

  def tag_to_location(conn, params) do
    tag = params["tag"]

    # establish a connection for the work we will be doing
    url = Application.get_env(:tanuki_backend, :couchdb_url)
    opts = Application.get_env(:tanuki_backend, :couchdb_opts)
    db_name = Application.get_env(:tanuki_backend, :database)
    server = :couchbeam.server_connection(url, opts)
    {:ok, db} = :couchbeam.open_or_create_db(server, db_name)

    # get all the docs with the tag
    {:ok, rows} = :couchbeam_view.fetch(db, {"assets", "by_tag"}, [{:key, tag}])
    count = List.foldl(rows, 0, fn(row, acc) ->
      doc_id = :couchbeam_doc.get_value("id", row)
      {:ok, doc} = :couchbeam.open_doc(db, doc_id)
      # if the doc has a location already, do not make any changes
      if TanukiBackend.get_field_value("location", doc) == nil do
        old_tags = :couchbeam_doc.get_value("tags", doc)
        new_tags = List.delete(old_tags, tag)
        new_doc = :couchbeam_doc.set_value("tags", new_tags, doc)
        new_doc = :couchbeam_doc.set_value("location", tag, new_doc)
        {:ok, _doc1} = :couchbeam.save_doc(db, new_doc)
        acc + 1
      else
        acc
      end
    end)

    conn
    |> put_flash(:info, "Updated #{count} documents")
    |> render(:index)
  end

  def sort_tags(conn, _params) do
    # establish a connection for the work we will be doing
    url = Application.get_env(:tanuki_backend, :couchdb_url)
    opts = Application.get_env(:tanuki_backend, :couchdb_opts)
    db_name = Application.get_env(:tanuki_backend, :database)
    server = :couchbeam.server_connection(url, opts)
    {:ok, db} = :couchbeam.open_or_create_db(server, db_name)

    count = :couchbeam_view.fold(fn(row, acc) ->
      doc_id = :couchbeam_doc.get_value("id", row)
      {:ok, doc} = :couchbeam.open_doc(db, doc_id)
      current_tags = TanukiBackend.get_field_value("tags", doc)
      if not is_nil(current_tags) do
        sorted_tags = Enum.sort(current_tags)
        if current_tags != sorted_tags do
          new_doc = :couchbeam_doc.set_value("tags", sorted_tags, doc)
          {:ok, _doc1} = :couchbeam.save_doc(db, new_doc)
          acc + 1
        else
          acc
        end
      else
        acc
      end
    end, 0, db, :all_docs)

    conn
    |> put_flash(:info, "Updated #{count} documents")
    |> render(:index)
  end

  def incoming(conn, _params) do
    {:ok, count} = GenServer.call(TanukiIncoming, :process_now)
    conn
    |> put_flash(:info, "Added #{count} assets")
    |> render(:index)
  end
end
