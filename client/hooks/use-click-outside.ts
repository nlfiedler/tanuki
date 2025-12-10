//
// Copyright (c) 2025 Nathan Fiedler
//
import { onCleanup, createEffect } from 'solid-js';

type Handler = (event: MouseEvent) => void;

function useClickOutside<T extends HTMLElement>(
  ref: () => T | undefined,
  handler: Handler
) {
  createEffect(() => {
    const element = ref();
    if (!element) return;

    const listener = (event: MouseEvent) => {
      if (!element || element.contains(event.target as Node)) {
        return;
      }
      handler(event);
    };

    document.addEventListener('mousedown', listener);

    onCleanup(() => {
      document.removeEventListener('mousedown', listener);
    });
  });
}

export default useClickOutside;
