//
// Copyright (c) 2026 Nathan Fiedler
//
import { type Accessor, createSignal } from 'solid-js';
import useClickOutside from '../hooks/use-click-outside.ts';
import galleryCardGridRaw from '../assets/icons/gallery-card-grid.svg?raw';
import galleryJustifiedRaw from '../assets/icons/gallery-justified.svg?raw';
import galleryMasonryRaw from '../assets/icons/gallery-masonry.svg?raw';
import gallerySquareGridRaw from '../assets/icons/gallery-square-grid.svg?raw';
import galleryThumbListRaw from '../assets/icons/gallery-thumb-list.svg?raw';

export type GalleryLayout =
  | 'cards'
  | 'rows'
  | 'masonry'
  | 'details'
  | 'squares';

const withCurrentColor = (svg: string) =>
  svg.replace('<svg ', '<svg fill="currentColor" ');

const cardGridIcon = withCurrentColor(galleryCardGridRaw);
const justifiedIcon = withCurrentColor(galleryJustifiedRaw);
const masonryIcon = withCurrentColor(galleryMasonryRaw);
const squareGridIcon = withCurrentColor(gallerySquareGridRaw);
const thumbListIcon = withCurrentColor(galleryThumbListRaw);

interface LayoutSelectorProps {
  selectedLayout: Accessor<GalleryLayout>;
  setLayout: (layout: GalleryLayout) => void;
}

function LayoutSelector(props: LayoutSelectorProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false);
  let dropdownRef: HTMLDivElement | undefined;
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  );
  const iconFor = (layout: GalleryLayout) => {
    switch (layout) {
      case 'rows': {
        return justifiedIcon;
      }
      case 'masonry': {
        return masonryIcon;
      }
      case 'details': {
        return thumbListIcon;
      }
      case 'squares': {
        return squareGridIcon;
      }
      default: {
        return cardGridIcon;
      }
    }
  };

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
          aria-controls="layout-menu"
          aria-label="Gallery layout"
        >
          <span class="icon" innerHTML={iconFor(props.selectedLayout())} />
        </button>
      </div>
      <div class="dropdown-menu" id="layout-menu" role="menu">
        <div class="dropdown-content">
          <a
            class="dropdown-item"
            role="menuitem"
            classList={{ 'is-active': props.selectedLayout() === 'cards' }}
            on:click={(_) => {
              props.setLayout('cards');
              setDropdownOpen(false);
            }}
          >
            <span class="icon-text">
              <span class="icon" innerHTML={cardGridIcon} />
              <span>Cards</span>
            </span>
          </a>
          <a
            class="dropdown-item"
            role="menuitem"
            classList={{ 'is-active': props.selectedLayout() === 'rows' }}
            on:click={(_) => {
              props.setLayout('rows');
              setDropdownOpen(false);
            }}
          >
            <span class="icon-text">
              <span class="icon" innerHTML={justifiedIcon} />
              <span>Rows</span>
            </span>
          </a>
          <a
            class="dropdown-item"
            role="menuitem"
            classList={{ 'is-active': props.selectedLayout() === 'masonry' }}
            on:click={(_) => {
              props.setLayout('masonry');
              setDropdownOpen(false);
            }}
          >
            <span class="icon-text">
              <span class="icon" innerHTML={masonryIcon} />
              <span>Columns</span>
            </span>
          </a>
          <a
            class="dropdown-item"
            role="menuitem"
            classList={{ 'is-active': props.selectedLayout() === 'details' }}
            on:click={(_) => {
              props.setLayout('details');
              setDropdownOpen(false);
            }}
          >
            <span class="icon-text">
              <span class="icon" innerHTML={thumbListIcon} />
              <span>Details</span>
            </span>
          </a>
          <a
            class="dropdown-item"
            role="menuitem"
            classList={{ 'is-active': props.selectedLayout() === 'squares' }}
            on:click={(_) => {
              props.setLayout('squares');
              setDropdownOpen(false);
            }}
          >
            <span class="icon-text">
              <span class="icon" innerHTML={squareGridIcon} />
              <span>Squares</span>
            </span>
          </a>
        </div>
      </div>
    </div>
  );
}

export default LayoutSelector;
