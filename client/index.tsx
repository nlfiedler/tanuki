//
// Copyright (c) 2025 Nathan Fiedler
//
/* @refresh reload */
import { render } from 'solid-js/web'
import { Router, Route } from '@solidjs/router'
import './main.scss'
import { ApolloProvider } from './ApolloProvider.jsx'
import Navbar from './Navbar.jsx'
import Home from './Home.jsx'
import Upload from './Upload.jsx'

function App(props: any) {
  return (
    <>
      <Navbar />
      {props.children}
    </>
  )
}

render(
  () => (
    <ApolloProvider>
      <Router root={App}>
        <Route path="" component={Home} />
        <Route path="/upload" component={Upload} />
        <Route path="*paramName" component={NotFound} />
      </Router>
    </ApolloProvider>
  ),
  document.getElementById('root')!
)

function NotFound() {
  return (
    <section class="section">
      <h1 class="title">Page not found</h1>
      <h2 class="subtitle">This is not the page you are looking for.</h2>
      <div class="content">
        <p>Try using the navigation options above.</p>
      </div>
    </section>
  )
}
