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
  let urlToShownPage = (url: ReasonReact.Router.url) =>
    switch (url.path) {
    | ["assets", id, "edit"] => EditRoute(id)
    | ["assets", id] => ShowRoute(id)
    | ["upload"] => UploadRoute
    | ["search"] => SearchRoute
    | [] => HomeRoute
    | _ => NotFoundRoute
    };
  let component = ReasonReact.reducerComponent("App");
  let make = _children => {
    ...component,
    initialState: () => {
      nowShowing:
        /*
         * Need to take the given URL in order to return to where we were
         * before; especially for uploading assets, in which the backend
         * redirects to the asset edit page. When that happens our application
         * is effectively reloading from scratch.
         */
        urlToShownPage(ReasonReact.Router.dangerouslyGetInitialUrl()),
    },
    reducer: (action, _state) =>
      switch (action) {
      | Navigate(page) => ReasonReact.Update({nowShowing: page})
      },
    didMount: self => {
      let token =
        ReasonReact.Router.watchUrl(url =>
          self.send(Navigate(urlToShownPage(url)))
        );
      self.onUnmount(() => ReasonReact.Router.unwatchUrl(token));
    },
    render: self => {
      let content =
        switch (self.state.nowShowing) {
        | HomeRoute => <Home.Component />
        | SearchRoute => <Search.Component />
        | ShowRoute(id) => <ShowAsset.Component assetId=id />
        | EditRoute(id) => <EditAsset.Component assetId=id />
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