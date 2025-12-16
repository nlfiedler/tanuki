//
// Copyright (c) 2025 Nathan Fiedler
//

/**
 * A simple circular buffer that will overwrite the oldest value if additional
 * elements are added while the buffer is already full.
 */
class CircularBuffer<T> {
  capacity: number;
  buffer: T[];
  head: number;
  tail: number;
  size: number;

  /** Constructs a circular buffer with the given capacity. */
  constructor(capacity: number) {
    this.capacity = capacity;
    this.buffer = Array.from({ length: capacity });
    this.head = 0;
    this.tail = 0;
    this.size = 0;
  }

  /** Adds an element to the buffer. */
  enqueue(item: T) {
    this.buffer[this.tail] = item;
    this.tail = (this.tail + 1) % this.capacity;
    if (this.size < this.capacity) {
      this.size++;
    } else {
      this.head = (this.head + 1) % this.capacity;
    }
  }

  /** Removes and returns the oldest element from the buffer. */
  dequeue(): T | undefined {
    if (this.size === 0) {
      return undefined;
    }
    const item = this.buffer[this.head];
    this.head = (this.head + 1) % this.capacity;
    this.size--;
    return item;
  }

  /** Returns the oldest element without removing it. */
  peek() {
    if (this.size === 0) {
      return;
    }
    return this.buffer[this.head];
  }

  /** Checks if the buffer is empty. */
  isEmpty() {
    return this.size === 0;
  }

  /** Checks if the buffer is full. */
  isFull() {
    return this.size === this.capacity;
  }

  /** Returns the number of elements in the buffer. */
  get length() {
    return this.size;
  }
}

export { CircularBuffer };
