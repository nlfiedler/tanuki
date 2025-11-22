//
// Copyright (c) 2025 Nathan Fiedler
//
import { createSignal } from 'solid-js'

function Home() {
  const [count, setCount] = createSignal(0)

  return (
    <div class="container">
      <div class="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count()}
        </button>
      </div>
    </div>
  )
}

export default Home
