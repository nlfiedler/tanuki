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
          thumbnailUrl
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
      let beforeYear = afterYear + 1;
      let yearToDate = (year: int) =>
        Js.Json.number(
          Js.Date.utcWithYM(~year=float_of_int(year), ~month=0.0, ()),
        );
      (Some(yearToDate(afterYear)), Some(yearToDate(beforeYear)));
    };
  {
    "after": afterTime,
    "before": beforeTime,
    "filename": None,
    "mimetype": None,
    "tags": Some(Belt.Set.String.toArray(state.selectedTags)),
    "locations": Some(Belt.Set.String.toArray(state.selectedLocations)),
  };
};

module HomeRe = {
  let component = ReasonReact.statelessComponent("HomeRe");
  let make = (~state: Redux.appState, ~dispatch, _children) => {
    ...component,
    render: _self => {
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
        ...{
             ({result}) =>
               switch (result) {
               | Loading => <div> {ReasonReact.string("Loading")} </div>
               | Error(error) =>
                 Js.log(error);
                 <div> {ReasonReact.string(error##message)} </div>;
               | Data(response) =>
                 <Thumbnails.Component
                   state
                   dispatch
                   search=response##search
                 />
               }
           }
      </QueryAssetsQuery>;
    },
  };
};

module SelectedProvider = {
  let lens = Reductive.Lens.make((state: Redux.appState) => state);
  let make = Reductive.Provider.createMake(Redux.store, lens);
};

module Component = {
  let component = ReasonReact.statelessComponent("Home");
  let make = _children => {
    ...component,
    render: _self =>
      <div>
        <Tags.Component />
        <Locations.Component />
        <Years.Component />
        <SelectedProvider component=HomeRe.make />
      </div>,
  };
};