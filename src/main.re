module App = {
  let component = ReasonReact.statelessComponent("App");
  let make = _children => {
    ...component,
    render: _self => <h3> {ReasonReact.string("Hello, world!")} </h3>,
  };
};

ReactDOMRe.renderToElementWithId(<App />, "main");