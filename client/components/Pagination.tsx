//
// Copyright (c) 2025 Nathan Fiedler
//
import type { Accessor, Setter } from 'solid-js';
import { For } from 'solid-js';
import { createSignal } from 'solid-js';
import useClickOutside from '../hooks/useClickOutside.js';

interface PaginationProps {
  lastPage: Accessor<number>;
  selectedPage: Accessor<number>;
  setSelectedPage: Setter<number>;
  pageSize: Accessor<number>;
  setPageSize: Setter<number>;
}

function Pagination(props: PaginationProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false);
  let dropdownRef: HTMLDivElement | undefined;
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  );

  return (
    <>
      <div class="level-item">
        <div class="field">
          <p class="control">
            <button
              class="button"
              disabled={props.selectedPage() === 1}
              on:click={(_) => props.setSelectedPage((p) => --p)}
            >
              <span class="icon">
                <i class="fas fa-angle-left"></i>
              </span>
            </button>
          </p>
        </div>
      </div>
      <div class="level-item">
        <span>{`Page ${props.selectedPage()} of ${props.lastPage()}`}</span>
      </div>
      <div class="level-item">
        <div class="field">
          <p class="control">
            <button
              class="button"
              disabled={props.selectedPage() >= props.lastPage()}
              on:click={(_) => props.setSelectedPage((p) => ++p)}
            >
              <span class="icon">
                <i class="fas fa-angle-right"></i>
              </span>
            </button>
          </p>
        </div>
      </div>
      <div class="level-item">
        <div class="field">
          <div
            class="dropdown is-right"
            ref={(el: HTMLDivElement) => (dropdownRef = el)}
            class:is-active={dropdownOpen()}
          >
            <div class="dropdown-trigger">
              <button
                class="button"
                on:click={(_) => setDropdownOpen((v) => !v)}
                aria-haspopup="true"
                aria-controls="dropdown-menu"
              >
                <span class="icon">
                  <i class="fa-solid fa-expand" aria-hidden="true"></i>
                </span>
              </button>
            </div>
            <div class="dropdown-menu" id="dropdown-menu" role="menu">
              <div class="dropdown-content">
                <For each={[18, 36, 54, 72]}>
                  {(size) => (
                    <a
                      class={
                        props.pageSize() === size
                          ? 'dropdown-item is-active'
                          : 'dropdown-item'
                      }
                      on:click={(_) => {
                        props.setPageSize(size);
                        props.setSelectedPage(1);
                        setDropdownOpen(false);
                      }}
                    >
                      {size.toString()}
                    </a>
                  )}
                </For>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export default Pagination;
