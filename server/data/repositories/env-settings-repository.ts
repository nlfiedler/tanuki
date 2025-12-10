//
// Copyright (c) 2025 Nathan Fiedler
//
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';

/**
 * Implementation of the settings repository that uses values read from the
 * process environment otherwise.
 */
class EnvSettingsRepository implements SettingsRepository {
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
    if (this._props.has(name)) {
      return this._props.get(name);
    }
    // fallback to reading directly from process environment to support a
    // container-based deployment that does not use a .env file
    return process.env[name];
  }

  /** @inheritdoc */
  getBool(name: string): boolean {
    return /true/i.test(this.get(name));
  }

  /** @inheritdoc */
  getInt(name: string, fallback: number): number {
    return Number.parseInt(this.get(name), 10) || fallback;
  }

  /** @inheritdoc */
  has(name: string): boolean {
    return this._props.has(name) || name in process.env;
  }

  /** @inheritdoc */
  set(name: string, value: any): void {
    this._props.set(name, value);
  }
}

export { EnvSettingsRepository };
