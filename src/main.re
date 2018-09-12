module GetTags = [%graphql
  {|
  query {
    tags {
      value
      count
    }
  }
|}
];

module GetTagsQuery = ReasonApollo.CreateQuery(GetTags);

module Tags = {
  let component = ReasonReact.statelessComponent("Tags");
  let make = _children => {
    ...component,
    render: _self =>
      <GetTagsQuery>
        ...{
             ({result}) =>
               switch (result) {
               | Loading => <div> {ReasonReact.string("Loading...")} </div>
               | Error(error) =>
                 Js.log(error);
                 <div> {ReasonReact.string(error##message)} </div>;
               | Data(response) =>
                 <ul>
                   {
                     ReasonReact.array(
                       Array.mapi(
                         (index, tag) =>
                           <li key={string_of_int(index)}>
                             {ReasonReact.string(tag##value)}
                           </li>,
                         response##tags,
                       ),
                     )
                   }
                 </ul>
               }
           }
      </GetTagsQuery>,
  };
};

module App = {
  let component = ReasonReact.statelessComponent("App");
  let make = _children => {
    ...component,
    render: _self =>
      <div> <h3> {ReasonReact.string("Tags")} </h3> <Tags /> </div>,
  };
};

ReactDOMRe.renderToElementWithId(
  <ReasonApollo.Provider client=Client.instance>
    <App />
  </ReasonApollo.Provider>,
  "main",
);