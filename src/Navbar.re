type state = {menuActive: bool};
type action =
  | ToggleMenu;
let component = ReasonReact.reducerComponent("Navbar");
let make = _children => {
  ...component,
  initialState: () => {menuActive: false},
  reducer: (action, state) =>
    switch (action) {
    | ToggleMenu => ReasonReact.Update({menuActive: !state.menuActive})
    },
  render: self => {
    let menuClassName =
      self.state.menuActive ? "navbar-menu is-active" : "navbar-menu";
    <nav id="navbar" className="navbar is-transparent" role="navigation">
      <div className="navbar-brand">
        <img src="/images/exposure-level.png" width="48" height="48" />
        <a
          role="button"
          className="navbar-burger"
          target="navMenu"
          onClick={_ => self.send(ToggleMenu)}>
          <span ariaHidden=true />
          <span ariaHidden=true />
          <span ariaHidden=true />
        </a>
      </div>
      <div className=menuClassName id="navMenu">
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
    </nav>;
  },
};