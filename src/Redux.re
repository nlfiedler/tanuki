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

let appReducer = (state, action) =>
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