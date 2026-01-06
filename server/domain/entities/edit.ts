//
// Copyright (c) 2025 Nathan Fiedler
//
import { Asset } from './asset.ts';
import { Location } from './location.ts';

/** Makes a modification on an asset (add tag, set location, etc). */
export interface Operation {
  /**
   * Possibly make a change to the given asset.
   *
   * @param asset - asset entity to be modified.
   * @returns true if the asset was changed, false otherwise.
   */
  perform(asset: Asset): boolean;
}

/** Adds a tag to an asset. */
export class TagAdd {
  name: string;

  constructor(name: string) {
    this.name = name;
  }

  perform(asset: Asset): boolean {
    if (!asset.tags.includes(this.name)) {
      asset.tags.push(this.name);
      return true;
    }
    return false;
  }
}

/** Removes a tag from an asset. */
export class TagRemove {
  name: string;

  constructor(name: string) {
    this.name = name;
  }

  perform(asset: Asset): boolean {
    if (asset.tags.includes(this.name)) {
      asset.tags = asset.tags.filter((t) => t !== this.name);
      return true;
    }
    return false;
  }
}

export enum LocationField {
  Label,
  City,
  Region
}

/** Clears a specific field of the location of the asset. */
export class LocationClearField {
  field: LocationField;

  constructor(field: LocationField) {
    this.field = field;
  }

  perform(asset: Asset): boolean {
    if (asset.location) {
      switch (this.field) {
        case LocationField.Label: {
          if (asset.location.label) {
            asset.location.label = null;
            return true;
          }
          break;
        }
        case LocationField.City: {
          if (asset.location.city) {
            asset.location.city = null;
            return true;
          }
          break;
        }
        case LocationField.Region: {
          if (asset.location.region) {
            asset.location.region = null;
            return true;
          }
          break;
        }
      }
    }
    return false;
  }
}

/** Sets the field of the location of the asset to a given value. */
export class LocationSetField {
  field: LocationField;
  value: string;

  constructor(field: LocationField, value: string) {
    this.field = field;
    this.value = value;
  }

  perform(asset: Asset): boolean {
    if (!asset.location) {
      asset.location = Location.fromRaw(null, null, null);
    }
    switch (this.field) {
      case LocationField.Label: {
        if (asset.location.label !== this.value) {
          asset.location.label = this.value;
          return true;
        }
        break;
      }
      case LocationField.City: {
        if (asset.location.city !== this.value) {
          asset.location.city = this.value;
          return true;
        }
        break;
      }
      case LocationField.Region: {
        if (asset.location.region !== this.value) {
          asset.location.region = this.value;
          return true;
        }
        break;
      }
    }
    return false;
  }
}

/** Clears the user date-time field of an asset. */
export class DatetimeClear {
  perform(asset: Asset): boolean {
    if (asset.userDate) {
      asset.userDate = null;
      return true;
    }
    return false;
  }
}

/** Sets the user date-time field of an asset. */
export class DatetimeSet {
  value: Date;

  constructor(value: Date) {
    this.value = value;
  }

  perform(asset: Asset): boolean {
    if (asset.bestDate() !== this.value) {
      asset.userDate = this.value;
      return true;
    }
    return false;
  }
}

/**
 * Adds a number of days to the user date-time field of an asset. The value may
 * be negative which will subtract days from the date-time.
 */
export class DatetimeAddDays {
  value: number;

  constructor(value: number) {
    this.value = value;
  }

  perform(asset: Asset): boolean {
    const newDate = asset.bestDate();
    newDate.setDate(newDate.getDate() + this.value);
    asset.userDate = newDate;
    return true;
  }
}
