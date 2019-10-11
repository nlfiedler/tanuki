//
// Copyright (c) 2019 Nathan Fiedler
//

// Waiting for support via React Hooks API, which then will
// appear in ReasonReact. Once those are ready, wrap the
// `content` element with this to catch errors.
//
// c.f. https://github.com/reasonml/reason-react/pull/247
// c.f. https://reactjs.org/docs/hooks-faq.html
// c.f. https://reactjs.org/docs/error-boundaries.html
//
// module ErrorBoundary = {
//   [@react.component]
//   let make = () => {
//     <h1> {React.string("Something went wrong")} </h1>;
//   };
// };

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
    <Redux.Provider store=Redux.appStore><App /></Redux.Provider>
  </ReasonApollo.Provider>,
  "main",
);