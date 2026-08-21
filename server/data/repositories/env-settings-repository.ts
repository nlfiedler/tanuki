//
// Copyright (c) 2025 Nathan Fiedler
//
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';

/**
 * Implementation of the settings repository that uses values read from the
 * process environment otherwise.
 */
class EnvSettingsRepository implements SettingsRepository {
  #props: Map<string, any>;

  constructor() {
    this.#props = new Map();
  }

  /**
   * Return a Map-based iterator of all name/value pairs.
   *
   * @returns iterator of name/value pairs.
   */
  entries(): object {
    return this.#props.entries();
  }

  /**
  @inheritdoc
  */
  get(name: string): any {
    if (this.#props.has(name)) {
      return this.#props.get(name);
    }
    // fallback to reading directly from process environment to support a
    // container-based deployment that does not use a .env file
    return process.env[name];
  }

  /**
  @inheritdoc
  */
  getBool(name: string): boolean {
    return /true/i.test(this.get(name));
  }

  /**
  @inheritdoc
  */
  getInt(name: string, fallback: number): number {
    return Math.trunc(Number(this.get(name))) || fallback;
  }

  /**
  @inheritdoc
  */
  getFloat(name: string, fallback: number): number {
    const value = Number(this.get(name));
    return Number.isNaN(value) ? fallback : value;
  }

  /**
  @inheritdoc
  */
  has(name: string): boolean {
    return this.#props.has(name) || Object.hasOwn(process.env, name);
  }

  /**
  @inheritdoc
  */
  set(name: string, value: any): void {
    this.#props.set(name, value);
  }
}

export { EnvSettingsRepository };
