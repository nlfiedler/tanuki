module Component = {
  let component = ReasonReact.statelessComponent("Search");
  let make = _children => {
    ...component,
    render: _self =>
      <div> <h3> {ReasonReact.string("Search Page")} </h3> </div>,
  };
};