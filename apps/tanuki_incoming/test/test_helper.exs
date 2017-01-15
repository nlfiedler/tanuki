ExUnit.start()

# Ensure CouchDB is running.
case :gen_tcp.connect('localhost', 5984, []) do
  {:ok, socket} ->
    :gen_tcp.close(socket)
  {:error, reason} ->
    Mix.raise "Cannot connect to CouchDB" <>
              " (http://localhost:5984):" <>
              " #{:inet.format_error(reason)}"
end

# Stop and unload our applications so the test code can configure
# everything dynamically, including creating temporary directories.
Application.stop(:tanuki_backend)
Application.stop(:tanuki_incoming)
Application.unload(:tanuki_backend)
Application.unload(:tanuki_incoming)
