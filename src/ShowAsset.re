module Component = {
  let component = ReasonReact.statelessComponent("ShowAsset");
  let make = _children => {
    ...component,
    render: _self =>
      <div> <h3> {ReasonReact.string("Asset Page")} </h3> </div>,
  };
};