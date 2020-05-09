//
// Copyright (c) 2020 Nathan Fiedler
//
/* The expected shape of the location from GraphQL. */
type t = {
  .
  "label": string,
  "count": int,
};

/* Name the query so the mutations can invoke in refetchQueries. */
module GetLocations = [%graphql
  {|
    query getAllLocations {
      locations {
        label
        count
      }
    }
  |}
];

module GetLocationsQuery = ReasonApollo.CreateQuery(GetLocations);

/*
 * Return locations that match several criteria.
 *
 * Returns the top 25 locations sorted by the number of assets they select, and then
 * sorted by the label. Locations that are currently selected by the user are always
 * included in the result.
 */
let selectTopLocations =
    (selectedLocations: Belt.Set.String.t, allLocations: array(t)) => {
  /* get the fully defined selected locations into a map keyed by name */
  let selectedLocationsMap =
    Array.fold_right(
      (location, coll) =>
        if (Belt.Set.String.has(selectedLocations, location##label)) {
          Belt.Map.String.set(coll, location##label, location);
        } else {
          coll;
        },
      allLocations,
      Belt.Map.String.empty,
    );
  /* keep the top 25 locations by count, merge with the selected */
  let sortedLocations = Array.copy(allLocations);
  Array.sort((a, b) => b##count - a##count, sortedLocations);
  let mergedMap =
    Array.fold_right(
      (location, coll) =>
        Belt.Map.String.set(coll, location##label, location),
      Array.sub(sortedLocations, 0, 25),
      selectedLocationsMap,
    );
  /* extract the map values and sort by name */
  let almostThere = Belt.Map.String.valuesToArray(mergedMap);
  Array.sort((a, b) => compare(a##label, b##label), almostThere);
  almostThere;
};

// React hooks require a stable function reference to work properly.
let stateSelector = (state: Redux.appState) => state.selectedLocations;

module Component = {
  type state = {showingAll: bool};
  type action =
    | ToggleAll;
  [@react.component]
  let make = () => {
    let reduxState = Redux.useSelector(stateSelector);
    let reduxDispatch = Redux.useDispatch();
    let (state, dispatch) =
      React.useReducer(
        (state, action) =>
          switch (action) {
          | ToggleAll => {showingAll: !state.showingAll}
          },
        {showingAll: false},
      );
    let allLocationsToggle = (locations: array(t)) =>
      if (state.showingAll) {
        <a
          className="tag is-light"
          href="#"
          title="Hide some locations"
          onClick={_ => dispatch(ToggleAll)}>
          {ReasonReact.string("<<")}
        </a>;
      } else if (Array.length(locations) <= 25) {
        ReasonReact.null;
      } else {
        <a
          className="tag is-light"
          href="#"
          title="Show all locations"
          onClick={_ => dispatch(ToggleAll)}>
          {ReasonReact.string(">>")}
        </a>;
      };
    let buildLocations = (locations: array(t)) => {
      let visibleLocations =
        if (state.showingAll || Array.length(locations) <= 25) {
          locations;
        } else {
          selectTopLocations(reduxState, locations);
        };
      Array.mapi(
        (index, location: t) => {
          let isSelected = Belt.Set.String.has(reduxState, location##label);
          let className = isSelected ? "tag is-dark" : "tag is-light";
          <a
            key={string_of_int(index)}
            className
            href="#"
            title={string_of_int(location##count)}
            onClick={_ =>
              reduxDispatch(Redux.ToggleLocation(location##label))
            }>
            {ReasonReact.string(location##label)}
          </a>;
        },
        visibleLocations,
      );
    };
    <GetLocationsQuery>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading locations...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          <div className="tags">
            <span className="tag is-info">
              {ReasonReact.string("Locations")}
            </span>
            {ReasonReact.array(buildLocations(response##locations))}
            {allLocationsToggle(response##locations)}
          </div>
        }
      }
    </GetLocationsQuery>;
  };
};