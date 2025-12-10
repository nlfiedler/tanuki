//
// Copyright (c) 2025 Nathan Fiedler
//
import { createSignal, createEffect } from 'solid-js';
import useClickOutside from '../hooks/use-click-outside.ts';

function ColorTheme() {
  const [theme, setTheme] = createSignal('light');
  const [dropdownOpen, setDropdownOpen] = createSignal(false);
  let dropdownRef: HTMLDivElement | undefined;
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  );
  createEffect(() => {
    document.documentElement.dataset.theme = dataForTheme(theme());
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('theme', theme());
    }
  });
  if (typeof localStorage !== 'undefined' && localStorage.getItem('theme')) {
    setTheme(localStorage.getItem('theme')!);
  } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    setTheme('dark');
  } else {
    setTheme('light');
  }

  return (
    <div
      class="dropdown is-right"
      ref={(el: HTMLDivElement) => (dropdownRef = el)}
      class:is-active={dropdownOpen()}
    >
      <div class="dropdown-trigger">
        <button
          class="button"
          on:click={() => setDropdownOpen((v) => !v)}
          aria-haspopup="true"
          aria-controls="dropdown-menu"
        >
          <span class="icon">
            <i class={classForTheme(theme())} aria-hidden="true"></i>
          </span>
        </button>
      </div>
      <div class="dropdown-menu" id="dropdown-menu" role="menu">
        <div class="dropdown-content">
          <a
            class={
              theme() == 'light' ? 'dropdown-item is-active' : 'dropdown-item'
            }
            on:click={() => {
              setTheme('light');
              setDropdownOpen(false);
            }}
          >
            <span class="icon">
              <i class="fa-solid fa-sun" aria-hidden="true"></i>
            </span>
            <span>Light</span>
          </a>
          <a
            class={
              theme() == 'dark' ? 'dropdown-item is-active' : 'dropdown-item'
            }
            on:click={() => {
              setTheme('dark');
              setDropdownOpen(false);
            }}
          >
            <span class="icon">
              <i class="fa-solid fa-moon" aria-hidden="true"></i>
            </span>
            <span>Dark</span>
          </a>
          <a
            class={
              theme() == 'auto' ? 'dropdown-item is-active' : 'dropdown-item'
            }
            on:click={() => {
              setTheme('auto');
              setDropdownOpen(false);
            }}
          >
            <span class="icon">
              <i class="fa-solid fa-desktop" aria-hidden="true"></i>
            </span>
            <span>System</span>
          </a>
        </div>
      </div>
    </div>
  );
}

// Return the appropriate icon CSS class for the current color theme.
function classForTheme(theme: string) {
  switch (theme) {
    case 'dark': {
      return 'fa-solid fa-moon';
    }
    case 'light': {
      return 'fa-solid fa-sun';
    }
    default: {
      return 'fa-solid fa-desktop';
    }
  }
}

// Return the appropriate data-theme value for the given color theme.
function dataForTheme(theme: string) {
  if (theme === 'light') {
    return 'light';
  } else if (theme === 'dark') {
    return 'dark';
  }
  if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark';
  }
  return 'light';
}

export default ColorTheme;
