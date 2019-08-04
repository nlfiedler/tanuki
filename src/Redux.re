//
// Copyright (c) 2018 Nathan Fiedler
//
type searchInputs = {
  tags: string,
  locations: string,
  afterDate: string,
  beforeDate: string,
  filename: string,
  mimetype: string,
};

type appAction =
  | ToggleTag(string)
  | ToggleLocation(string)
  | ToggleYear(int)
  | Paginate(int)
  | SaveSearch(searchInputs);

type appState = {
  selectedTags: Belt.Set.String.t,
  selectedLocations: Belt.Set.String.t,
  selectedYear: option(int),
  pageNumber: int,
  savedSearch: searchInputs,
};

let appReducer = (state: appState, action: appAction) =>
  switch (action) {
  | ToggleTag(label) => {
      ...state,
      pageNumber: 1,
      selectedTags:
        Belt.Set.String.has(state.selectedTags, label)
          ? Belt.Set.String.remove(state.selectedTags, label)
          : Belt.Set.String.add(state.selectedTags, label),
    }
  | ToggleLocation(label) => {
      ...state,
      pageNumber: 1,
      selectedLocations:
        Belt.Set.String.has(state.selectedLocations, label)
          ? Belt.Set.String.remove(state.selectedLocations, label)
          : Belt.Set.String.add(state.selectedLocations, label),
    }
  | ToggleYear(year) => {
      ...state,
      pageNumber: 1,
      selectedYear:
        Belt.Option.eq(state.selectedYear, Some(year), (a, b) => a == b)
          ? None : Some(year),
    }
  | Paginate(page) => {...state, pageNumber: page}
  | SaveSearch(inputs) => {...state, savedSearch: inputs}
  };

module Thunk = {
  /**
   * Thunks allow for defining actions outside of this global definition,
   * such as within components.
   */
  type thunk('state) = ..;

  type thunk('state) +=
    | Thunk((Reductive.Store.t(thunk('state), 'state) => unit));
};

module Middleware = {
  /**
   * Middleware API:
   * store: gives you access to state before and after the dispatch
   * next: the next function to call in the chain. Any middleware can be async.
   * action: this allows you to look for specific actions to operate on
   * return value can be used by the middleware that called you (optional)
   */

  /**
   * Logs the action before dispatching and the new state after.
   */
  let logger = (store, next, action) => {
    Js.log(action);
    let returnValue = next(action);
    Js.log(Reductive.Store.getState(store));
    returnValue;
  };

  /**
   * Listens for a specific action and calls that function.
   * Allows for async actions.
   */
  let thunk = (store, next, action) =>
    switch (action) {
    | Thunk.Thunk(func) => func(store)
    | _ => next(action)
    };
};

// Not currently using thunked actions, but if we did, this requires
// that we define actions within the ReduxThunk.thunk type, like so:
//
// type ReduxThunk.thunk(_) +=
//   | StringAction (stringAction)
//   | CounterAction (action);
//
// Additionally, the `action` in ReductiveContext.Make() would need
// to be of type ReduxThunk.thunk(appState) to reflect the change.
//
// let storeLogger = (store, next: 'a => 'b) =>
//   Middleware.thunk(store) @@ Middleware.logger(store) @@ next;
let storeLogger = (store, next) => Middleware.logger(store) @@ next;

let appStore =
  Reductive.Store.create(
    ~reducer=appReducer,
    ~preloadedState={
      selectedTags: Belt.Set.String.empty,
      selectedLocations: Belt.Set.String.empty,
      selectedYear: None,
      pageNumber: 1,
      savedSearch: {
        tags: "",
        locations: "",
        afterDate: "",
        beforeDate: "",
        filename: "",
        mimetype: "",
      },
    },
    ~enhancer=storeLogger,
    (),
  );

include ReductiveContext.Make({
  type state = appState;
  type action = appAction;
  let store = appStore;
});