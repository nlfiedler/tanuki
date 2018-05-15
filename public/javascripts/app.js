//
// Add Elm to the page, as needed.
//
/* global Elm */
const elmMain = document.getElementById('elm-main')
if (elmMain) {
  // detect if the browser has drag and drop support
  var draggable = 'draggable' in document.createElement('span')
  Elm.Main.embed(elmMain, {draggable: draggable})
}
