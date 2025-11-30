//
// Copyright (c) 2025 Nathan Fiedler
//
import { type Accessor, type Setter, createSignal, createEffect, onCleanup } from 'solid-js';

function useLocalStorage<T>(key: string, initialValue: T): [Accessor<T>, Setter<T>] {
  const readValue = () => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.warn(`Error reading localStorage key "${key}":`, error);
      return initialValue;
    }
  };

  const [value, setValue] = createSignal<T>(readValue());

  // Update local storage when the signal changes
  createEffect(() => {
    try {
      window.localStorage.setItem(key, JSON.stringify(value()));
    } catch (error) {
      console.warn(`Error setting localStorage key "${key}":`, error);
    }
  });

  // Listen for changes from other tabs/windows
  const handleStorageChange = () => {
    setValue(readValue());
  };
  window.addEventListener('storage', handleStorageChange);
  onCleanup(() => {
    window.removeEventListener('storage', handleStorageChange);
  });

  return [value, setValue];
}

export default useLocalStorage;
