# 0001 — `has:` Predicate in Query Language

## Summary

Add a new `has:` predicate to the query language so users can filter assets
by whether a given field is populated. For example, `has:caption` matches
assets that have a non-empty caption, and `has:location` matches assets that
have any location data attached.

## Motivation

Until now the query language could match assets by tag, location, date range,
media type, and free-text terms, but there was no way to ask "show me every
asset that has X set" (or, conversely, "every asset missing X" via the
existing `not` operator). This is useful for finding assets that still need
metadata (e.g. `not has:caption`) and for narrowing searches to assets that
have user-supplied data (e.g. `has:userDate`).

## Design

A new predicate `has:<field>` is parsed alongside the existing predicates
(`tag:`, `loc:`, `before:`, `after:`, `is:`, etc.). It evaluates to true when
the named field on the `Asset` entity has a populated value.

"Populated" is defined as:

- not `null` and not `undefined`
- if a string, length > 0
- if an array, length > 0

Field name matching is tolerant of casing and separators: `userDate`,
`user_date`, `user-date`, and `USERDATE` all resolve to the same `Asset`
property. This is implemented by lower-casing both sides and stripping `-`
and `_` before comparison. Unknown field names match nothing rather than
raising an error, which keeps the predicate safe to use against future or
optional fields.

## Changes

### `server/domain/usecases/query.ts`

- New `HasConstraint` class implementing the `Constraint` interface. Its
  constructor normalises the field name; `matches(asset)` walks the asset's
  own keys, finds one whose normalised name matches, and returns true iff
  the value passes the populated check above.
- `buildPredicate` extended with a `has` keyword arm that constructs a
  `HasConstraint` from the predicate's argument.

### `test/domain/usecases/query.test.ts`

- New test case `should parse query and match by has` covering:
  - A bare asset (only the required fields set): `has:caption`,
    `has:location`, `has:userDate`, `has:tags` all return false;
    `has:filename` returns true; `has:nonexistent` returns false.
  - A populated asset with caption, location, tags, userDate, and
    originalDate: each corresponding `has:` predicate returns true.
  - Field-name normalisation: `has:userDate`, `has:user_date`,
    `has:user-date`, `has:USERDATE`, and `has:original_date` all match.
  - Composition with existing operators: `has:caption and tag:kitten`
    matches the populated asset and not the bare one.

## Compatibility

Purely additive. Existing queries continue to parse and evaluate as before;
no schema, storage, or API changes are required.
