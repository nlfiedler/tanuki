//
// Copyright (c) 2025 Nathan Fiedler
//
import { A } from '@solidjs/router';
import tanukiPng from '../assets/tanuki.png';
import AssetCount from './asset-count.tsx';
import ColorTheme from './color-theme.tsx';

function Navbar() {
  return (
    <nav class="navbar" role="navigation" aria-label="main navigation">
      <div class="navbar-brand">
        <img class="navbar-item" src={tanukiPng} width="80" height="80" />
        <a
          role="button"
          class="navbar-burger"
          aria-label="menu"
          aria-expanded="false"
          data-target="navbarMenu"
        >
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
        </a>
      </div>

      <div id="navbarMenu" class="navbar-menu">
        <div class="navbar-start">
          <A class="navbar-item" href="/">
            Browse
          </A>

          <A class="navbar-item" href="/search">
            Search
          </A>

          <A class="navbar-item" href="/upload">
            Upload
          </A>

          <A class="navbar-item" href="/pending">
            Pending
          </A>

          <A class="navbar-item" href="/edit">
            Edit
          </A>
        </div>

        <div class="navbar-end">
          <div class="navbar-item">
            <ColorTheme />
          </div>
          <div class="navbar-item">
            <AssetCount />
          </div>
        </div>
      </div>
    </nav>
  );
}

export default Navbar;
