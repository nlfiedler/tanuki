let component = ReasonReact.statelessComponent("Navbar");
let make = _children => {
  ...component,
  render: _self =>
    <nav id="navbar" className="navbar is-transparent" role="navigation">
      <div className="navbar-brand">
        <img src="/images/tanuki.png" width="520" height="100" />
      </div>
      <div className="navbar-menu" id="navMenu">
        <div className="navbar-end">
          <a
            className="navbar-item"
            onClick={_ => ReasonReact.Router.push("/")}>
            <span className="icon"> <i className="fas fa-lg fa-home" /> </span>
          </a>
          <a
            className="navbar-item"
            onClick={_ => ReasonReact.Router.push("/upload")}>
            <span className="icon">
              <i className="fas fa-lg fa-upload" />
            </span>
          </a>
          <a
            className="navbar-item"
            onClick={_ => ReasonReact.Router.push("/search")}>
            <span className="icon">
              <i className="fas fa-lg fa-search" />
            </span>
          </a>
          <a className="navbar-item" href="/graphiql">
            <span className="icon">
              <i className="fas fa-lg fa-search-plus" />
            </span>
          </a>
        </div>
      </div>
    </nav>,
};