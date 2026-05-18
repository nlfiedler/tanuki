//
// Copyright (c) 2026 Nathan Fiedler
//
import { createSignal, onCleanup, onMount, Show } from 'solid-js';

// Pixels of upward scroll required before the button appears. Avoids the button
// flickering on tiny scroll jitter from trackpads or rubber-banding.
const UP_THRESHOLD = 40;
// Minimum scroll position for the button to appear at all.
const MIN_OFFSET = 200;

function BackToTop() {
  const [visible, setVisible] = createSignal(false);
  let lastY = 0;
  let upAccum = 0;

  const onScroll = () => {
    const y = window.scrollY;
    const delta = y - lastY;
    if (delta < 0) {
      upAccum += -delta;
      if (y > MIN_OFFSET && upAccum >= UP_THRESHOLD) {
        setVisible(true);
      }
    } else if (delta > 0) {
      upAccum = 0;
      setVisible(false);
    }
    if (y <= MIN_OFFSET) {
      setVisible(false);
    }
    lastY = y;
  };

  onMount(() => {
    lastY = window.scrollY;
    window.addEventListener('scroll', onScroll, { passive: true });
    onCleanup(() => window.removeEventListener('scroll', onScroll));
  });

  const scrollToTop = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
    setVisible(false);
  };

  return (
    <Show when={visible()}>
      <button
        type="button"
        class="button"
        on:click={scrollToTop}
        style={{
          position: 'fixed',
          top: '4rem',
          left: '50%',
          transform: 'translateX(-50%)',
          'z-index': '30',
          'box-shadow': '0 2px 8px rgba(0, 0, 0, 0.25)'
        }}
      >
        <span class="icon">
          <i class="fa-solid fa-arrow-up" aria-hidden="true"></i>
        </span>
        <span>Back to top</span>
      </button>
    </Show>
  );
}

export default BackToTop;
