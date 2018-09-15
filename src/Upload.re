module Component = {
  let component = ReasonReact.statelessComponent("Upload");
  let make = _children => {
    ...component,
    render: _self =>
      <div> <h3> {ReasonReact.string("Upload Page")} </h3> </div>,
  };
};