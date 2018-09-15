/* module StringProvider = {
     let lens = Reductive.Lens.make((a: appState) => a.content);
     let make = Reductive.Provider.createMake(store, lens);
   }; */
module TagProvider = {
  let lens = Reductive.Lens.make((state: Redux.appState) => state.selectedTags);
  let make = Reductive.Provider.createMake(Redux.store, lens);
};

module App = {
  let component = ReasonReact.statelessComponent("App");
  let make = _children => {
    ...component,
    render: _self =>
      <div className="container">
        <Navbar />
        <main role="main">
          <TagProvider component=Tags.Component.make />
        </main>
      </div>,
  };
};

ReactDOMRe.renderToElementWithId(
  <ReasonApollo.Provider client=Client.instance>
    <App />
  </ReasonApollo.Provider>,
  "main",
);