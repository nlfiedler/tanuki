/* TODO: need navigation in the a tags below... */
let navbar =
  <nav id="navbar" className="navbar is-transparent" role="navigation">
    <div className="navbar-brand">
      <img src="/images/tanuki.png" width="520" height="100" />
    </div>
    <div className="navbar-menu" id="navMenu">
      <div className="navbar-end">
        <a className="navbar-item">
          <span className="icon"> <i className="fas fa-lg fa-home" /> </span>
        </a>
        <a className="navbar-item">
          <span className="icon"> <i className="fas fa-lg fa-upload" /> </span>
        </a>
        <a className="navbar-item">
          <span className="icon"> <i className="fas fa-lg fa-search" /> </span>
        </a>
        <a className="navbar-item" href="/graphiql">
          <span className="icon">
            <i className="fas fa-lg fa-search-plus" />
          </span>
        </a>
      </div>
    </div>
  </nav>;

module App = {
  let component = ReasonReact.statelessComponent("App");
  let make = _children => {
    ...component,
    render: _self =>
      <div className="container">
        navbar
        <main role="main"> <Tags.Component /> </main>
      </div>,
  };
};

ReactDOMRe.renderToElementWithId(
  <ReasonApollo.Provider client=Client.instance>
    <App />
  </ReasonApollo.Provider>,
  "main",
);