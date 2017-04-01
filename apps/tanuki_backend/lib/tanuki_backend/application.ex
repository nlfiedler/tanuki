defmodule TanukiBackend.Application do
  @moduledoc false
  use Application

  def start(_type, _args) do
    import Supervisor.Spec, warn: false

    children = [
      # Arguments include the initial server state and the arguments to the
      # GenServer.start_link/3, which the server module invokes.
      worker(TanukiBackend.Server, [[], [name: TanukiDatabase]]),
    ]
    opts = [strategy: :one_for_one, name: TanukiBackend.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
