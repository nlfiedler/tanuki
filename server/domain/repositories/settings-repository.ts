//
// Copyright (c) 2025 Nathan Fiedler
//

/**
 * Defines the interface for retrieving application settings which consist of
 * name/value pairs. The names are strings and values may be structured data.
 */
interface SettingsRepository {
  /**
   * Return the value of the named setting, possibly undefined.
   *
   * @param name - name of property to retrieve.
   * @returns property value if found.
   */
  get(name: string): any;

  /**
   * Return the boolean value of the named setting, default is false.
   *
   * @param name - name of property to retrieve.
   * @returns boolean value of the property if found, otherwise false.
   */
  getBool(name: string): boolean;

  /**
   * Return the integer value of the named setting, default is fallback.
   *
   * @param name - name of property to retrieve.
   * @param fallback - value to return if property not found.
   * @returns value of property or fallback it not found.
   */
  getInt(name: string, fallback: number): number;

  /**
   * Return true if the settings repository contains the named setting.
   *
   * @param name - name of property to locate.
   * @returns true if property if found, otherwise false.
   */
  has(name: string): boolean;

  /**
   * Set the value for the named setting.
   *
   * @param name - name of property to define.
   * @param value - value for the named property.
   */
  set(name: string, value: any): void;
}

export { type SettingsRepository };
