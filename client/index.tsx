//
// Copyright (c) 2025 Nathan Fiedler
//
/* @refresh reload */
import { render } from 'solid-js/web';
import { Router, Route } from '@solidjs/router';
import './assets/main.scss';
import { ApolloProvider } from './apollo-provider.tsx';
import Navbar from './components/navbar.tsx';
import AssetDetails from './pages/details.tsx';
import Home from './pages/home.tsx';
import Pending from './pages/pending.tsx';
import Search from './pages/search.tsx';
import Upload from './pages/upload.tsx';

function App(props: any) {
  return (
    <>
      <Navbar />
      {props.children}
    </>
  );
}

render(
  () => (
    <ApolloProvider>
      <Router root={App}>
        <Route path="" component={Home} />
        <Route path="/pending" component={Pending} />
        <Route path="/search" component={Search} />
        <Route path="/upload" component={Upload} />
        <Route path="/asset/:id" component={AssetDetails} />
        <Route path="*paramName" component={NotFound} />
      </Router>
    </ApolloProvider>
  ),
  document.querySelector('#root')!
);

function NotFound() {
  return (
    <section class="section">
      <h1 class="title">Page not found</h1>
      <h2 class="subtitle">This is not the page you are looking for.</h2>
      <div class="content">
        <p>Try using the navigation options above.</p>
      </div>
    </section>
  );
}
