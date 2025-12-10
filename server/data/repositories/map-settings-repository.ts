//
// Copyright (c) 2025 Nathan Fiedler
//
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';

/**
 * Implementation of the settings repository backed by a Map.
 */
class MapSettingsRepository implements SettingsRepository {
  _props: Map<string, any>;

  constructor() {
    this._props = new Map();
  }

  /**
   * Return a Map-based iterator of all name/value pairs.
   *
   * @returns iterator of name/value pairs.
   */
  entries(): object {
    return this._props.entries();
  }

  /** @inheritdoc */
  get(name: string): any {
    return this._props.get(name);
  }

  /** @inheritdoc */
  getBool(name: string): boolean {
    return /true/i.test(this._props.get(name));
  }

  /** @inheritdoc */
  getInt(name: string, fallback: number): number {
    return Number.parseInt(this._props.get(name), 10) || fallback;
  }

  /** @inheritdoc */
  has(name: string): boolean {
    return this._props.has(name);
  }

  /** @inheritdoc */
  set(name: string, value: any): void {
    this._props.set(name, value);
  }

  /** @inheritdoc */
  delete(name: string): void {
    this._props.delete(name);
  }

  /** Removes all elements from the underlying map. */
  clear(): void {
    this._props.clear();
  }
}

export { MapSettingsRepository };
