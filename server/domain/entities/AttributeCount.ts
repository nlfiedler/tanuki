//
// Copyright (c) 2025 Nathan Fiedler
//

/**
 * An asset attribute label and the number of assets with this attribute.
 */
class AttributeCount {
  label: string;
  count: number;

  constructor(label: string, count: number) {
    this.label = label;
    this.count = count;
  }
}

export { AttributeCount };
