module TagProvider = {
  let lens =
    Reductive.Lens.make((state: Redux.appState) => state.selectedTags);
  let make = Reductive.Provider.createMake(Redux.store, lens);
};

module Component = {
  let component = ReasonReact.statelessComponent("Home");
  let make = _children => {
    ...component,
    render: _self =>
      <div> <TagProvider component=Tags.Component.make /> </div>,
  };
};