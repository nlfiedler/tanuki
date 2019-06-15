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

module YearsRe = {
  let component = ReasonReact.statelessComponent("YearsRe");
  let make = (~state: option(int), ~dispatch, _children) => {
    let buildYears = years =>
      Array.mapi(
        (index, year) => {
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
    {
      ...component,
      render: _self =>
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
        </GetYearsQuery>,
    };
  };
};

module YearsProvider = {
  let make =
    Reductive.Lense.createMake(
      ~lense=(s: Redux.appState) => s.selectedYear,
      Redux.store,
    );
};

module Component = {
  let component = ReasonReact.statelessComponent("Years");
  let make = _children => {
    ...component,
    render: _self => <YearsProvider component=YearsRe.make />,
  };
};