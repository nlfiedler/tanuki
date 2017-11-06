//
// Copyright (c) 2017 Nathan Fiedler
//

function allTags () {
  return [
    {'tag': 'cat', 'count': 2},
    {'tag': 'dog', 'count': 5},
    {'tag': 'kitten', 'count': 100},
    {'tag': 'puppy', 'count': 40}
  ]
}

function allLocations () {
  return [
    {'location': 'dublin', 'count': 6},
    {'location': 'hawaii', 'count': 4},
    {'location': 'piedmont', 'count': 5},
    {'location': 'vancouver', 'count': 7}
  ]
}

function allYears () {
  return [
    {'year': 1986, 'count': 1},
    {'year': 1992, 'count': 1},
    {'year': 1994, 'count': 1},
    {'year': 2002, 'count': 9}
  ]
}

module.exports = {
  allTags,
  allLocations,
  allYears
}
