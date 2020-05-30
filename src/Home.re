//
// Copyright (c) 2020 Nathan Fiedler
//
module CountAssets = [%graphql {|
    query {
      count
    }
  |}];

module CountAssetsQuery = ReasonApollo.CreateQuery(CountAssets);

/*
 * The home page shows selectors for the various attributes, and based on
 * user selection, shows thumbnails for matching assets.
 */
module QueryAssets = [%graphql
  {|
    query Search($params: SearchParams!, $pageSize: Int, $offset: Int) {
      search(params: $params, count: $pageSize, offset: $offset) {
        results {
          id
          datetime
          filename
          location
          mimetype
        }
        count
      }
    }
  |}
];

module QueryAssetsQuery = ReasonApollo.CreateQuery(QueryAssets);

let makeQueryParams = (state: Redux.appState) => {
  let (afterTime, beforeTime) =
    if (Belt.Option.isNone(state.selectedYear)) {
      (None, None);
    } else {
      let afterYear = Belt.Option.getExn(state.selectedYear);
      let beforeYear = int_of_string(afterYear) + 1;
      let yearToDate = (year: string) =>
        Js.Json.string(year ++ "-01-01T00:00:00Z");
      (
        Some(yearToDate(afterYear)),
        Some(yearToDate(string_of_int(beforeYear))),
      );
    };
  {
    "after": afterTime,
    "before": beforeTime,
    "filename": None,
    "mimetype": None,
    "tags": Some(Belt.Set.String.toArray(state.selectedTags)),
    "locations": Some(Belt.Set.String.toArray(state.selectedLocations)),
    "sortField": None,
    "sortOrder": None,
  };
};

// React hooks require a stable function reference to work properly.
let stateSelector = (state: Redux.appState) => state;

module HomeRe = {
  [@react.component]
  let make = () => {
    let state = Redux.useSelector(stateSelector);
    let dispatch = Redux.useDispatch();
    let offset = (state.pageNumber - 1) * Thumbnails.pageSize;
    let queryParams = makeQueryParams(state);
    let query =
      QueryAssets.make(
        ~params=queryParams,
        ~pageSize=Thumbnails.pageSize,
        ~offset,
        (),
      );
    <QueryAssetsQuery variables=query##variables>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          <Thumbnails.Component state dispatch search=response##search />
        }
      }
    </QueryAssetsQuery>;
  };
};

module Component = {
  [@react.component]
  let make = () => {
    <CountAssetsQuery>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          if (response##count > 0) {
            <div>
              <Tags.Component />
              <Locations.Component />
              <Years.Component />
              <HomeRe />
            </div>;
          } else {
            <div>
              <p>
                {ReasonReact.string("Use the")}
                <span className="icon"> <i className="fas fa-upload" /> </span>
                {ReasonReact.string("upload feature to add assets.")}
              </p>
            </div>;
          }
        }
      }
    </CountAssetsQuery>;
  };
};