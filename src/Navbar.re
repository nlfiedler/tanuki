//
// Copyright (c) 2019 Nathan Fiedler
//
type state = {menuActive: bool};
type action =
  | ToggleMenu;
[@react.component]
let make = () => {
  let (state, dispatch) =
    React.useReducer(
      (state, action) =>
        switch (action) {
        | ToggleMenu => {menuActive: !state.menuActive}
        },
      {menuActive: false},
    );
  let menuClassName =
    state.menuActive ? "navbar-menu is-active" : "navbar-menu";
  <nav id="navbar" className="navbar is-transparent" role="navigation">
    <div className="navbar-brand">
      <img src="/images/exposure-level.png" width="48" height="48" />
      <a
        role="button"
        className="navbar-burger"
        target="navMenu"
        onClick={_ => dispatch(ToggleMenu)}>
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
               "onClick": _ => ReasonReactRouter.push("/"),
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
               "onClick": _ => ReasonReactRouter.push("/upload"),
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
               "onClick": _ => ReasonReactRouter.push("/search"),
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
};