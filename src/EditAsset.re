module Component = {
  let component = ReasonReact.statelessComponent("EditAsset");
  let make = _children => {
    ...component,
    render: _self =>
      <div> <h3> {ReasonReact.string("Asset Edit Page")} </h3> </div>,
  };
};