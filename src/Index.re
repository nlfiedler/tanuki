//
// Copyright (c) 2018 Nathan Fiedler
//
module App = {
  [@react.component]
  let make = () => {
    let url = ReasonReactRouter.useUrl();
    let content =
      switch (url.path) {
      | ["assets", id, "edit"] => <EditAsset.Component assetId=id />
      | ["assets", id] => <ShowAsset.Component assetId=id />
      | ["upload"] => <Upload.Component />
      | ["search"] => <Search.Component />
      | [] => <Home.Component />
      | _ => <NotFound.Component />
      };
    <div className="container">
      <Navbar />
      <main role="main"> content </main>
    </div>;
  };
};

ReactDOMRe.renderToElementWithId(
  <ReasonApollo.Provider client=Client.instance>
    <App />
  </ReasonApollo.Provider>,
  "main",
);