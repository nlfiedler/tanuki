/*
 * The home page shows selectors for the various attributes, and based on
 * user selection, shows thumbnails for matching assets.
 */
module QueryAssets = [%graphql
  {|
    query Search($params: SearchParams!, $pageSize: Int) {
      search(params: $params, count: $pageSize) {
        results {
          id
          datetime
          filename
          location
        }
        count
      }
    }
  |}
];

module QueryAssetsQuery = ReasonApollo.CreateQuery(QueryAssets);

module TagsProvider = {
  let lens =
    Reductive.Lens.make((state: Redux.appState) => state.selectedTags);
  let make = Reductive.Provider.createMake(Redux.store, lens);
};

module SelectedProvider = {
  /* TODO: at some point should narrow the lens to the "selected" fields */
  let lens = Reductive.Lens.make((state: Redux.appState) => state);
  let make = Reductive.Provider.createMake(Redux.store, lens);
};

let makeQueryParams = (state: Redux.appState) => {
  "after": None,
  "before": None,
  "filename": None,
  "locations": None,
  "mimetype": None,
  "tags": Some(Belt.Set.String.toArray(state.selectedTags)),
};
/*
 function fromSelections (selections) {
   const locations = selections.locations.map(item => item.label)
   const tags = selections.tags.map(item => item.label)
   const years = selections.years.map(item => Number.parseInt(item.label))
   let afterTime = null
   let beforeTime = null
   if (years.length > 0) {
     afterTime = new Date(years[0], 0).getTime()
     beforeTime = new Date(years[0] + 1, 0).getTime()
   }
   return {
     locations,
     tags,
     after: afterTime,
     before: beforeTime
   }
 }
 */

module Main = {
  let component = ReasonReact.statelessComponent("Main");
  let make = (~state: Redux.appState, ~dispatch, _children) => {
    ...component,
    render: _self => {
      ignore(dispatch);
      let queryParams = makeQueryParams(state);
      let query =
        QueryAssets.make(
          ~params=queryParams,
          ~pageSize=Thumbnails.pageSize,
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
                 <Thumbnails.Component search=response##search />
               }
           }
      </QueryAssetsQuery>;
    },
  };
};

module Component = {
  let component = ReasonReact.statelessComponent("Home");
  let make = _children => {
    ...component,
    render: _self =>
      <div>
        <TagsProvider component=Tags.Component.make />
        <SelectedProvider component=Main.make />
      </div>,
  };
};