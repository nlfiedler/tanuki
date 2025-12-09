//
// Copyright (c) 2025 Nathan Fiedler
//
/* @refresh reload */
import { render } from 'solid-js/web';
import { Router, Route } from '@solidjs/router';
import './assets/main.scss';
import { ApolloProvider } from './ApolloProvider.tsx';
import Navbar from './components/Navbar.tsx';
import AssetDetails from './pages/Details.tsx';
import Home from './pages/Home.tsx';
import Pending from './pages/Pending.tsx';
import Upload from './pages/Upload.tsx';

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
        <Route path="/upload" component={Upload} />
        <Route path="/asset/:id" component={AssetDetails} />
        <Route path="*paramName" component={NotFound} />
      </Router>
    </ApolloProvider>
  ),
  document.getElementById('root')!
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
