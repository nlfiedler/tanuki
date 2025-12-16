//
// Copyright (c) 2025 Nathan Fiedler
//

/**
 * A simple circular buffer that grows its capacity if additional elements are
 * added when the buffer is already full.
 */
class ArrayDeque<T> {
  capacity: number;
  growthFactor: number;
  buffer: Array<T | undefined>;
  head: number;
  tail: number;
  size: number;

  /**
   * Create a new ArrayDeque with a capacity of 4 and growth factor of 2.
   *
   * @param initialCapacity number of elements to receive without growing.
   * @param growthFactor multiplier used to grow the array when it becomes full.
   */
  constructor(initialCapacity = 4, growthFactor = 2) {
    this.capacity = initialCapacity;
    this.growthFactor = growthFactor;
    this.buffer = Array.from({ length: this.capacity });
    this.head = 0;
    this.tail = 0;
    this.size = 0;
  }

  /**
   * Grows the buffer capacity and copies the elements to the new buffer.
   */
  grow() {
    const newCapacity = this.capacity * this.growthFactor;
    const newBuffer = Array.from<T>({ length: newCapacity });
    for (let i = 0; i < this.size; i++) {
      const oldIndex = (this.head + i) % this.capacity;
      newBuffer[i] = this.buffer[oldIndex]!;
    }
    this.buffer = newBuffer;
    this.capacity = newCapacity;
    this.head = 0;
    this.tail = this.size;
  }

  /**
   * Adds an element to the buffer. Grows the buffer if it is full.
   *
   * @param item - the item to add.
   */
  enqueue(item: T) {
    if (this.size === this.capacity) {
      this.grow();
    }
    this.buffer[this.tail] = item;
    this.tail = (this.tail + 1) % this.capacity;
    this.size++;
  }

  /**
   * Removes and returns the front element from the buffer.
   *
   * @returns The front element, or undefined if empty.
   */
  dequeue(): T | undefined {
    if (this.size === 0) {
      return;
    }
    const item = this.buffer[this.head];
    this.buffer[this.head] = undefined;
    this.head = (this.head + 1) % this.capacity;
    this.size--;
    return item;
  }

  /**
   * Returns the number of elements in the buffer.
   */
  get length(): number {
    return this.size;
  }

  /**
   * Peeks at the front element without removing it.
   */
  peek(): T | undefined {
    if (this.size === 0) {
      return;
    }
    return this.buffer[this.head];
  }
}

export { ArrayDeque };
