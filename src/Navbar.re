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
          {ReactDOMRe.createElement(
             "a",
             ~props=
               ReactDOMRe.objToDOMProps({
                 "className": "navbar-item tooltip is-tooltip-bottom",
                 "data-tooltip": "Browse assets",
                 "onClick": _ => ReasonReact.Router.push("/"),
               }),
             [|
               <span className="icon">
                 <i className="fas fa-lg fa-home" />
               </span>,
             |],
           )}
          {ReactDOMRe.createElement(
             "a",
             ~props=
               ReactDOMRe.objToDOMProps({
                 "className": "navbar-item tooltip is-tooltip-bottom",
                 "data-tooltip": "Upload assets",
                 "onClick": _ => ReasonReact.Router.push("/upload"),
               }),
             [|
               <span className="icon">
                 <i className="fas fa-lg fa-upload" />
               </span>,
             |],
           )}
          {ReactDOMRe.createElement(
             "a",
             ~props=
               ReactDOMRe.objToDOMProps({
                 "className": "navbar-item tooltip is-tooltip-bottom",
                 "data-tooltip": "Search assets",
                 "onClick": _ => ReasonReact.Router.push("/search"),
               }),
             [|
               <span className="icon">
                 <i className="fas fa-lg fa-search" />
               </span>,
             |],
           )}
          {ReactDOMRe.createElement(
             "a",
             ~props=
               ReactDOMRe.objToDOMProps({
                 "className": "navbar-item tooltip is-tooltip-bottom",
                 "data-tooltip": "Advanced search",
                 "href": "/graphql",
               }),
             [|
               <span className="icon">
                 <i className="fas fa-lg fa-search-plus" />
               </span>,
             |],
           )}
        </div>
      </div>
    </nav>;
  },
};