//
// Copyright (c) 2020 Nathan Fiedler
//

String validateMediaType(String val) {
  if (val.contains(RegExp(r'\s+'))) {
    return 'Media type must not contain white space';
  }
  if (val.contains('/')) {
    final List<String> parts = val.split('/');
    if (parts.length > 2) {
      return 'Media type may have only one slash (/)';
    }
    if (parts.any((e) => e.trim().isEmpty)) {
      return 'Media type must have a type and subtype';
    }
  } else {
    return 'Media type must contain a slash (/)';
  }
  return null;
}
