module App = {
  type route =
    | HomeRoute
    | SearchRoute
    | ShowRoute(string)
    | EditRoute(string)
    | UploadRoute
    | NotFoundRoute;
  type action =
    | Navigate(route);
  type state = {nowShowing: route};
  let component = ReasonReact.reducerComponent("App");
  let make = _children => {
    ...component,
    initialState: () => {nowShowing: HomeRoute},
    reducer: (action, _state) =>
      switch (action) {
      | Navigate(page) => ReasonReact.Update({nowShowing: page})
      },
    didMount: self => {
      let token =
        ReasonReact.Router.watchUrl(url =>
          switch (url.path) {
          | ["assets", id, "edit"] => self.send(Navigate(EditRoute(id)))
          | ["assets", id] => self.send(Navigate(ShowRoute(id)))
          | ["upload"] => self.send(Navigate(UploadRoute))
          | ["search"] => self.send(Navigate(SearchRoute))
          | [] => self.send(Navigate(HomeRoute))
          | _ => self.send(Navigate(NotFoundRoute))
          }
        );
      self.onUnmount(() => ReasonReact.Router.unwatchUrl(token));
    },
    render: self => {
      let content =
        switch (self.state.nowShowing) {
        | HomeRoute => <Home.Component />
        | SearchRoute => <Search.Component />
        | ShowRoute(id) => <ShowAsset.Component assetId=id />
        | EditRoute(_id) => <EditAsset.Component />
        | UploadRoute => <Upload.Component />
        | NotFoundRoute => <NotFound.Component />
        };
      <div className="container">
        <Navbar />
        <main role="main"> content </main>
      </div>;
    },
  };
};

ReactDOMRe.renderToElementWithId(
  <ReasonApollo.Provider client=Client.instance>
    <App />
  </ReasonApollo.Provider>,
  "main",
);