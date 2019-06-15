/* Name the query so the mutations can invoke in refetchQueries. */
module GetLocations = [%graphql
  {|
    query getAllLocations {
      locations {
        value
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
let selectTopLocations = (selectedLocations, allLocations) => {
  /* get the fully defined selected locations into a map keyed by name */
  let selectedLocationsMap =
    Array.fold_right(
      (location, coll) =>
        if (Belt.Set.String.has(selectedLocations, location##value)) {
          Belt.Map.String.set(coll, location##value, location);
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
        Belt.Map.String.set(coll, location##value, location),
      Array.sub(sortedLocations, 0, 25),
      selectedLocationsMap,
    );
  /* extract the map values and sort by name */
  let almostThere = Belt.Map.String.valuesToArray(mergedMap);
  Array.sort((a, b) => compare(a##value, b##value), almostThere);
  almostThere;
};

module LocationsRe = {
  type state = {showingAll: bool};
  type action =
    | ToggleAll;
  let component = ReasonReact.reducerComponent("LocationsRe");
  let make = (~state: Belt.Set.String.t, ~dispatch, _children) => {
    let allLocationsToggle = (state, send, locations) =>
      if (state.showingAll) {
        <a
          className="tag is-light"
          href="#"
          title="Hide some locations"
          onClick={_ => send(ToggleAll)}>
          {ReasonReact.string("<<")}
        </a>;
      } else if (Array.length(locations) <= 25) {
        ReasonReact.null;
      } else {
        <a
          className="tag is-light"
          href="#"
          title="Show all locations"
          onClick={_ => send(ToggleAll)}>
          {ReasonReact.string(">>")}
        </a>;
      };
    let buildLocations = (myState, locations) => {
      let visibleLocations =
        if (myState.showingAll || Array.length(locations) <= 25) {
          locations;
        } else {
          selectTopLocations(state, locations);
        };
      Array.mapi(
        (index, location) => {
          let isSelected = Belt.Set.String.has(state, location##value);
          let className = isSelected ? "tag is-dark" : "tag is-light";
          <a
            key={string_of_int(index)}
            className
            href="#"
            title={string_of_int(location##count)}
            onClick={_ => dispatch(Redux.ToggleLocation(location##value))}>
            {ReasonReact.string(location##value)}
          </a>;
        },
        visibleLocations,
      );
    };
    {
      ...component,
      initialState: () => {showingAll: false},
      reducer: action =>
        switch (action) {
        | ToggleAll => (
            state => ReasonReact.Update({showingAll: !state.showingAll})
          )
        },
      render: self =>
        <GetLocationsQuery>
          ...{({result}) =>
            switch (result) {
            | Loading =>
              <div> {ReasonReact.string("Loading locations...")} </div>
            | Error(error) =>
              Js.log(error);
              <div> {ReasonReact.string(error##message)} </div>;
            | Data(response) =>
              <div className="tags">
                <span className="tag is-info">
                  {ReasonReact.string("Locations")}
                </span>
                {ReasonReact.array(
                   buildLocations(self.state, response##locations),
                 )}
                {allLocationsToggle(
                   self.state,
                   self.send,
                   response##locations,
                 )}
              </div>
            }
          }
        </GetLocationsQuery>,
    };
  };
};

module LocationsProvider = {
  let make =
    Reductive.Lense.createMake(
      ~lense=(s: Redux.appState) => s.selectedLocations,
      Redux.store,
    );
};

module Component = {
  let component = ReasonReact.statelessComponent("Locations");
  let make = _children => {
    ...component,
    render: _self => <LocationsProvider component=LocationsRe.make />,
  };
};