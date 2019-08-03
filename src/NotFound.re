module Component = {
  let component = ReasonReact.statelessComponent("NotFound");
  let make = _children => {
    ...component,
    render: _self =>
      <div className="container">
        <article className="message is-warning">
          <div className="message-header">
            {ReasonReact.string("Warning")}
          </div>
          <div className="message-body">
            <div
              className="content"
              style={ReactDOMRe.Style.make(~fontFamily="monospace", ())}>
              {ReasonReact.string("Page not found")}
            </div>
          </div>
        </article>
        <a
          className="button is-link"
          onClick={_ => ReasonReact.Router.push("/")}>
          {ReasonReact.string("Back to home")}
        </a>
      </div>,
  };
};