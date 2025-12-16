//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { AsyncQueue } from 'tanuki/server/shared/collections/async-queue.ts';

describe('AsyncQueue', function () {
  test('should enqueue/dequeue capacity elements without blocking', async function () {
    // arrange
    const sut = new AsyncQueue<number>(8);
    // act
    for (let value = 0; value < 8; value++) {
      await sut.enqueue(value);
    }
    for (let value = 0; value < 8; value++) {
      const actual = await sut.dequeue();
      expect(actual).toEqual(value)
    }
    // assert
    expect(sut.length).toEqual(0);
    expect(sut.isEmpty).toBeTrue();
    expect(sut.closed).toBeFalse();
    sut.close();
    expect(sut.closed).toBeTrue();
  });

  test('should quickly clear the queue with slow producer, fast consumer', async function () {
    // arrange
    const sut = new AsyncQueue<number>(8);
    // act
    producer(sut, 10, 32);
    // the consumer will empty the queue and wait for the producer
    consumer(sut, 5);
    await new Promise<void>((resolve) => {
      const interval = setInterval(() => {
        if (sut.done) {
          clearInterval(interval);
          // give the consumer a chance to exit properly although not really
          // necessary when it is keeping up with the producer
          setTimeout(() => resolve());
        }
      }, 10);
    });
    // assert
    expect(sut.length).toEqual(0);
  });

  test('should clear the queue with equal producer/consumer timing', async function () {
    // arrange
    const sut = new AsyncQueue<number>(8);
    // act
    producer(sut, 5, 32);
    consumer(sut, 5);
    await new Promise<void>((resolve) => {
      const interval = setInterval(() => {
        if (sut.done) {
          clearInterval(interval);
          // give the consumer a chance to exit properly although not really
          // necessary when it is keeping up with the producer
          setTimeout(() => resolve());
        }
      }, 10);
    });
    // assert
    expect(sut.length).toEqual(0);
  });

  test('should eventually clear the queue with fast producer, slow consumer', async function () {
    // arrange
    const sut = new AsyncQueue<number>(8);
    // act
    producer(sut, 5, 32);
    // the producer will fill the queue and wait for the consumer
    consumer(sut, 10);
    await new Promise<void>((resolve) => {
      const interval = setInterval(() => {
        if (sut.done) {
          clearInterval(interval);
          // give the consumer a chance to exit properly
          setTimeout(() => resolve());
        }
      }, 20);
    });
    // assert
    expect(sut.length).toEqual(0);
  });
});

/** Will enqueue integers from 1 to limit. */
function producer(queue: AsyncQueue<number>, delay: number, limit: number) {
  // complicated logic to prevent the interval firing again while the async
  // function is still waiting; make sure the interval is cleared eventually
  let value = 0;
  let isRunning = false;
  const interval = setInterval(async () => {
    if (!isRunning) {
      isRunning = true;
      try {
        value++;
        if (value > limit) {
          // signal the consumers that the queue is closed
          queue.close();
          clearInterval(interval);
        } else {
          await queue.enqueue(value);
        }
      } finally {
        isRunning = false;
      }
    }
  }, delay);
}

/** Will continue consuming from the queue until it is closed and empty. */
function consumer(queue: AsyncQueue<number>, delay: number) {
  // complicated logic to prevent the interval firing again while the async
  // function is still waiting; make sure the interval is cleared eventually
  let isRunning = false;
  const interval = setInterval(async () => {
    if (!isRunning) {
      isRunning = true;
      try {
        await queue.dequeue();
      } catch {
        // exit once the queue is closed
        clearInterval(interval);
      } finally {
        isRunning = false;
      }
    }
  }, delay);
}
