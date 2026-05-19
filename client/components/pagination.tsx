//
// Copyright (c) 2025 Nathan Fiedler
//
import type { Accessor, Setter } from 'solid-js';
import { For } from 'solid-js';
import { createSignal } from 'solid-js';
import useClickOutside from '../hooks/use-click-outside.ts';

interface PaginationProps {
  lastPage: Accessor<number>;
  selectedPage: Accessor<number>;
  setSelectedPage: Setter<number>;
  pageSize: Accessor<number>;
  setPageSize: Setter<number>;
}

function Pagination(props: PaginationProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false);
  const [editingPage, setEditingPage] = createSignal(false);
  const [pageInput, setPageInput] = createSignal('');
  let dropdownRef: HTMLDivElement | undefined;
  let pageInputRef: HTMLInputElement | undefined;
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  );

  function openPageEditor() {
    setPageInput(String(props.selectedPage()));
    setEditingPage(true);
    queueMicrotask(() => {
      pageInputRef?.focus();
      pageInputRef?.select();
    });
  }

  function commitPageInput() {
    const parsed = Number.parseInt(pageInput(), 10);
    if (Number.isFinite(parsed)) {
      const clamped = Math.min(Math.max(parsed, 1), props.lastPage());
      props.setSelectedPage(clamped);
    }
    setEditingPage(false);
  }

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
        {editingPage() ? (
          <form
            on:submit={(e) => {
              e.preventDefault();
              commitPageInput();
            }}
          >
            <div class="field is-grouped is-align-items-center mb-0">
              <p class="control">
                <input
                  ref={(el: HTMLInputElement) => (pageInputRef = el)}
                  class="input is-small"
                  style="width: 4rem"
                  type="number"
                  min="1"
                  max={props.lastPage()}
                  value={pageInput()}
                  on:input={(e) => setPageInput(e.currentTarget.value)}
                  on:blur={() => setEditingPage(false)}
                  on:keydown={(e) => {
                    if (e.key === 'Escape') {
                      e.preventDefault();
                      setEditingPage(false);
                    }
                  }}
                />
              </p>
              <span>{` of ${props.lastPage()}`}</span>
            </div>
          </form>
        ) : (
          <button
            class="button is-text"
            disabled={props.lastPage() <= 1}
            on:click={(_) => openPageEditor()}
          >
            {`Page ${props.selectedPage()} of ${props.lastPage()}`}
          </button>
        )}
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
                      role="menuitem"
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
