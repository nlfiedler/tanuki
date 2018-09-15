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

/*
 * Return tags that match several criteria.
 *
 * Returns the top 25 tags sorted by the number of assets they select, and then
 * sorted by the label. Tags that are currently selected by the user are always
 * included in the result.
 */
let selectTopTags = (selectedTags, allTags) => {
  /* get the fully defined selected tags into a map keyed by name */
  let selectedTagsMap =
    Array.fold_right(
      (tag, coll) =>
        if (Belt.Set.String.has(selectedTags, tag##value)) {
          Belt.Map.String.set(coll, tag##value, tag);
        } else {
          coll;
        },
      allTags,
      Belt.Map.String.empty,
    );
  /* keep the top 25 tags by count, merge with the selected */
  Array.sort((a, b) => a##count - b##count, Array.copy(allTags));
  let mergedMap =
    Array.fold_right(
      (tag, coll) => Belt.Map.String.set(coll, tag##value, tag),
      Array.sub(allTags, 0, 25),
      selectedTagsMap,
    );
  /* extract the map values and sort by name */
  let almostThere = Belt.Map.String.valuesToArray(mergedMap);
  Array.sort((a, b) => compare(a##value, b##value), almostThere);
  almostThere;
};

module Component = {
  type state = {showingAll: bool};
  type action =
    | ToggleAll;
  let component = ReasonReact.reducerComponent("Tags");
  let make = (~state: Belt.Set.String.t, ~dispatch, _children) => {
    let allTagsToggle = (state, send, tags) =>
      if (state.showingAll) {
        <a
          className="tag is-light"
          href="#"
          title="Hide some tags"
          onClick={_ => send(ToggleAll)}>
          {ReasonReact.string("<<")}
        </a>;
      } else if (Array.length(tags) <= 25) {
        ReasonReact.null;
      } else {
        <a
          className="tag is-light"
          href="#"
          title="Show all tags"
          onClick={_ => send(ToggleAll)}>
          {ReasonReact.string(">>")}
        </a>;
      };
    let buildTags = (myState, tags) => {
      let visibleTags =
        if (myState.showingAll || Array.length(tags) <= 25) {
          tags;
        } else {
          selectTopTags(state, tags);
        };
      Array.mapi(
        (index, tag) => {
          let isSelected = Belt.Set.String.has(state, tag##value);
          let className = isSelected ? "tag is-dark" : "tag is-light";
          <a
            key={string_of_int(index)}
            className
            href="#"
            title={string_of_int(tag##count)}
            onClick={_ => dispatch(Redux.ToggleTag(tag##value))}>
            {ReasonReact.string(tag##value)}
          </a>;
        },
        visibleTags,
      );
    };
    {
      ...component,
      initialState: () => {showingAll: false},
      reducer: action =>
        switch (action) {
        | ToggleAll => (
            state => ReasonReact.Update({showingAll: !state.showingAll})
          )
        },
      render: self =>
        <GetTagsQuery>
          ...{
               ({result}) =>
                 switch (result) {
                 | Loading =>
                   <div> {ReasonReact.string("Loading tags...")} </div>
                 | Error(error) =>
                   Js.log(error);
                   <div> {ReasonReact.string(error##message)} </div>;
                 | Data(response) =>
                   <div className="tags">
                     <span className="tag is-info">
                       {ReasonReact.string("Tags")}
                     </span>
                     {
                       ReasonReact.array(
                         buildTags(self.state, response##tags),
                       )
                     }
                     {allTagsToggle(self.state, self.send, response##tags)}
                   </div>
                 }
             }
        </GetTagsQuery>,
    };
  };
};