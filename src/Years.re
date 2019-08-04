//
// Copyright (c) 2018 Nathan Fiedler
//
/* The expected shape of the year from GraphQL. */
type t = {
  .
  "value": int,
  "count": int,
};

/* Name the query so the mutations can invoke in refetchQueries. */
module GetYears = [%graphql
  {|
    query getAllYears {
      years {
        value
        count
      }
    }
  |}
];

module GetYearsQuery = ReasonApollo.CreateQuery(GetYears);

// React hooks require a stable function reference to work properly.
let stateSelector = (state: Redux.appState) => state.selectedYear;

module Component = {
  [@react.component]
  let make = () => {
    let state = Redux.useSelector(stateSelector);
    let dispatch = Redux.useDispatch();
    let buildYears = (years: array(t)) =>
      Array.mapi(
        (index, year: t) => {
          let isSelected =
            Belt.Option.eq(state, Some(year##value), (a, b) => a == b);
          let className = isSelected ? "tag is-dark" : "tag is-light";
          <a
            key={string_of_int(index)}
            className
            href="#"
            title={string_of_int(year##count)}
            onClick={_ => dispatch(Redux.ToggleYear(year##value))}>
            {ReasonReact.string(string_of_int(year##value))}
          </a>;
        },
        years,
      );
    <GetYearsQuery>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading years...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          <div className="tags">
            <span className="tag is-info">
              {ReasonReact.string("Years")}
            </span>
            {ReasonReact.array(buildYears(response##years))}
          </div>
        }
      }
    </GetYearsQuery>;
  };
};