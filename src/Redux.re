type appAction =
  | ToggleTag(string)
  | ToggleLocation(string)
  | ToggleYear(int);

/* TODO: when a toggle action occurs, need to re-fetch assets */
/* type ReduxThunk.thunk(_) +=
   | StringAction(stringAction)
   | CounterAction(counterAction); */

type appState = {
  selectedTags: Belt.Set.String.t,
  selectedLocations: Belt.Set.String.t,
  selectedYears: Belt.Set.Int.t,
};

let appReducer = (state, action) =>
  switch (action) {
  | ToggleTag(label) => {
      ...state,
      selectedTags:
        Belt.Set.String.has(state.selectedTags, label) ?
          Belt.Set.String.remove(state.selectedTags, label) :
          Belt.Set.String.add(state.selectedTags, label),
    }
  | ToggleLocation(label) => {
      ...state,
      selectedLocations:
        Belt.Set.String.has(state.selectedLocations, label) ?
          Belt.Set.String.remove(state.selectedLocations, label) :
          Belt.Set.String.add(state.selectedLocations, label),
    }
  | ToggleYear(year) => {
      ...state,
      selectedYears:
        Belt.Set.Int.has(state.selectedYears, year) ?
          Belt.Set.Int.remove(state.selectedYears, year) :
          Belt.Set.Int.add(state.selectedYears, year),
    }
  };

/*
 * let thunkedLogger = (store, next) =>
 *   Middleware.thunk(store) @@ Middleware.logger(store) @@ next;
 */
let storeLogger = (store, next) => Middleware.logger(store) @@ next;

let store =
  Reductive.Store.create(
    ~reducer=appReducer,
    ~preloadedState={
      selectedTags: Belt.Set.String.empty,
      selectedLocations: Belt.Set.String.empty,
      selectedYears: Belt.Set.Int.empty,
    },
    ~enhancer=storeLogger,
    (),
  );
