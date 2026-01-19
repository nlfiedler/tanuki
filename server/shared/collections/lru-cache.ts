//
// Copyright (c) 2026 Nathan Fiedler
//

/** Node for the doubly-linked list. */
class Node<K, V> {
  key: K;
  value: V;
  weight: number;
  prev: Node<K, V> | null;
  next: Node<K, V> | null;

  constructor(key: K, value: V, weight: number) {
    this.key = key;
    this.value = value;
    this.weight = weight;
    this.next = null;
    this.prev = null;
  }
}

/** A doubly-linked list for maintaining the order of items in the LRU cache. */
class DoublyLinkedList<K, V> {
  head: Node<K, V> | null = null;
  tail: Node<K, V> | null = null;

  /** Add the node to the head of the list. */
  addHead(node: Node<K, V>) {
    if (this.head === null) {
      this.head = node;
      this.tail = node;
    } else {
      node.next = this.head;
      this.head!.prev = node;
      this.head = node;
    }
  }

  /** Remove the node from the list. */
  removeNode(node: Node<K, V>) {
    if (node === this.head && node === this.tail) {
      this.head = null;
      this.tail = null;
    } else if (node === this.head) {
      this.head = node.next;
      this.head!.prev = null;
    } else if (node === this.tail) {
      this.tail = node.prev;
      this.tail!.next = null;
    } else {
      node.prev!.next = node.next;
      node.next!.prev = node.prev;
    }
    node.next = null;
    node.prev = null;
  }

  /** Remove the tail (LRU) node. */
  removeTail() {
    if (!this.tail) {
      return null;
    }
    const removedNode = this.tail;
    this.removeNode(removedNode);
    return removedNode;
  }

  /** Move an existing node to the head (MRU). */
  moveToHead(node: Node<K, V>) {
    this.removeNode(node);
    this.addHead(node);
  }
}

/**
 * LRU (least-recently-used) cache backed by a map and a doubly-linked list, in
 * which each item has an associated weight. When the combined weight exceeds
 * the defined maximum for the cache, the oldest items are removed.
 */
class LRUCache<K, V> {
  capacity: number;
  weight: number;
  cache: Map<K, Node<K, V>>;
  list: DoublyLinkedList<K, V>;

  /**
   * Create an LRUCache with the given maximum weight.
   *
   * @param capacity - maximum weight of the combined items.
   */
  constructor(capacity: number) {
    this.capacity = capacity;
    this.weight = 0;
    this.cache = new Map();
    this.list = new DoublyLinkedList();
  }

  get(key: K): V | undefined {
    const node = this.cache.get(key);
    if (node !== undefined) {
      this.list.moveToHead(node);
      return node.value;
    }
    return undefined;
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  /**
   * Adds the item to the cache. If an item with the same key exists, its value
   * will be replaced by the new value. Similarly, the overall weight will be
   * adjusted to reflect the weight of this new item.
   */
  set(key: K, value: V, weight = 1) {
    const existingNode = this.cache.get(key);
    if (existingNode === undefined) {
      // Add new item
      const newNode = new Node(key, value, weight);
      this.cache.set(key, newNode);
      this.list.addHead(newNode);
      this.weight += weight;
    } else {
      // Update existing item
      this.weight -= existingNode.weight;
      existingNode.value = value;
      existingNode.weight = weight;
      this.list.moveToHead(existingNode);
      this.weight += weight;
    }

    // Eviction policy: remove LRU items by weight until capacity is met,
    // possibly leaving the cache empty if the last item is too large
    while (this.weight > this.capacity) {
      const lruNode = this.list.removeTail();
      if (lruNode) {
        this.cache.delete(lruNode.key);
        this.weight -= lruNode.weight;
      }
    }
  }

  clear() {
    this.weight = 0;
    this.cache = new Map();
    this.list = new DoublyLinkedList();
  }

  get length() {
    return this.cache.size;
  }

  get currentWeight() {
    return this.weight;
  }
}

export { LRUCache };
