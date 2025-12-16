//
// Copyright (c) 2025 Nathan Fiedler
//
import { ArrayDeque } from './array-deque.ts';
import { CircularBuffer } from './circular-buffer.ts';

/** That which waits to place items into the queue when it has capacity. */
type Sender<T> = {
  value: T;
  resolve: (value: unknown) => void;
  reject: (err: any) => void;
};

/** That which waits for items to become available in the queue. */
type Receiver<T> = {
  resolve: (value: T) => void;
  reject: (err: any) => void;
};

/**
 * A simple Promise-based blocking queue with back pressure. That is, both the
 * enqueue and dequeue operations may wait asynchronously for conditions to
 * change such that the operation can proceed. In principle this is similiar to
 * buffered channels in Go.
 */
export class AsyncQueue<T> {
  private items: CircularBuffer<T>;
  private sendQueue: ArrayDeque<Sender<T>>;
  private recvQueue: ArrayDeque<Receiver<T>>;
  private isClosed: boolean;

  /**
   * Create a queue with the given capacity for items.
   *
   * @param capacity size of the item queue.
   */
  constructor(capacity: number) {
    this.items = new CircularBuffer(capacity);
    this.sendQueue = new ArrayDeque();
    this.recvQueue = new ArrayDeque();
    this.isClosed = false;
  }

  /**
   * Adds an item at the end of a queue. If the queue was empty and there are
   * any pending receivers from `dequeue()`, the first one will receive the
   * item. If the queue is full, this call asynchronously waits for space to
   * become available. If the queue has been closed, an error is thrown.
   *
   * @param value - item to be added to the back of the queue.
   */
  async enqueue(value: T) {
    if (this.isClosed) {
      throw new Error('queue has been closed');
    }
    if (this.recvQueue.length > 0) {
      // a receiver is waiting, immediately send this value
      const receiver = this.recvQueue.dequeue()!;
      receiver.resolve(value);
    } else if (this.items.isFull()) {
      // buffer is full, place the sender on a waiting list
      return new Promise((resolve, reject) => {
        this.sendQueue.enqueue({ value, resolve, reject });
      });
    } else {
      // buffer is not full and no receivers are waiting
      this.items.enqueue(value);
    }
  }

  /**
   * Attempts to consume an item from the queue in FIFO order, waiting until
   * there is one, unless the queue is closed.
   *
   * @returns the value from the front of the queue.
   */
  async dequeue(): Promise<T> {
    if (this.items.length > 0) {
      // If buffer has data, return it
      const value = this.items.dequeue()!;
      if (this.sendQueue.length > 0) {
        // If a sender is waiting, unblock it and move its value to buffer
        const sender = this.sendQueue.dequeue()!;
        this.items.enqueue(sender.value);
        sender.resolve(null);
      }
      return value;
    }
    if (this.sendQueue.length > 0) {
      // If a sender is waiting, directly receive its value
      const sender = this.sendQueue.dequeue()!;
      sender.resolve(null);
      return sender.value;
    }
    if (this.closed) {
      throw new Error('queue has been closed');
    }
    // No data and no waiting sender, wait for a sender
    return new Promise((resolve, reject) => {
      this.recvQueue.enqueue({ resolve, reject });
    });
  }

  /**
   * Switch to dequeue-only operation, no more items can be added. Any receivers
   * will be resolved with the remaining items, after which they will be
   * rejected with an error. All waiting senders will be rejected with an error.
   */
  close() {
    this.isClosed = true;
    // unblock all waiting receivers with either a value or an error
    while (this.recvQueue.length > 0) {
      const receiver = this.recvQueue.dequeue()!;
      if (this.items.length > 0) {
        receiver.resolve(this.items.dequeue()!);
      } else {
        receiver.reject(new Error('queue has been closed'));
      }
    }
    // unblock all waiting senders with an error
    while (this.sendQueue.length > 0) {
      const sender = this.sendQueue.dequeue()!;
      sender.reject(Promise.reject(new Error('queue closed while sending')));
    }
  }

  /** Checks if the queue is empty. */
  get isEmpty(): boolean {
    return this.items.isEmpty();
  }

  /** Checks if the queue is full. */
  get isFull(): boolean {
    return this.items.isFull();
  }

  /** Returns the current number of items in the queue. */
  get length(): number {
    return this.items.length;
  }

  /** Returns true if the queue has been closed and all items have been dequeued. */
  get done(): boolean {
    return this.isClosed && this.items.length === 0;
  }

  /** Returns true if the queue has been closed. */
  get closed(): boolean {
    return this.isClosed;
  }
}
