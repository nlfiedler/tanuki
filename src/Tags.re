//
// Copyright (c) 2020 Nathan Fiedler
//
/* The expected shape of the tag from GraphQL. */
type t = {
  .
  "label": string,
  "count": int,
};

/* Name the query so the mutations can invoke in refetchQueries. */
module GetTags = [%graphql
  {|
    query getAllTags {
      tags {
        label
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
let selectTopTags = (selectedTags: Belt.Set.String.t, allTags: array(t)) => {
  /* get the fully defined selected tags into a map keyed by name */
  let selectedTagsMap =
    Array.fold_right(
      (tag, coll) =>
        if (Belt.Set.String.has(selectedTags, tag##label)) {
          Belt.Map.String.set(coll, tag##label, tag);
        } else {
          coll;
        },
      allTags,
      Belt.Map.String.empty,
    );
  /* keep the top 25 tags by count, merge with the selected */
  let sortedTags = Array.copy(allTags);
  Array.sort((a, b) => b##count - a##count, sortedTags);
  let mergedMap =
    Array.fold_right(
      (tag, coll) => Belt.Map.String.set(coll, tag##label, tag),
      Array.sub(sortedTags, 0, 25),
      selectedTagsMap,
    );
  /* extract the map values and sort by name */
  let almostThere = Belt.Map.String.valuesToArray(mergedMap);
  Array.sort((a, b) => compare(a##label, b##label), almostThere);
  almostThere;
};

// React hooks require a stable function reference to work properly.
let stateSelector = (state: Redux.appState) => state.selectedTags;

module Component = {
  type state = {showingAll: bool};
  type action =
    | ToggleAll;
  [@react.component]
  let make = () => {
    let reduxState = Redux.useSelector(stateSelector);
    let reduxDispatch = Redux.useDispatch();
    let (state, dispatch) =
      React.useReducer(
        (state, action) =>
          switch (action) {
          | ToggleAll => {showingAll: !state.showingAll}
          },
        {showingAll: false},
      );
    let allTagsToggle = (tags: array(t)) =>
      if (state.showingAll) {
        <a
          className="tag is-light"
          href="#"
          title="Hide some tags"
          onClick={_ => dispatch(ToggleAll)}>
          {ReasonReact.string("<<")}
        </a>;
      } else if (Array.length(tags) <= 25) {
        ReasonReact.null;
      } else {
        <a
          className="tag is-light"
          href="#"
          title="Show all tags"
          onClick={_ => dispatch(ToggleAll)}>
          {ReasonReact.string(">>")}
        </a>;
      };
    let buildTags = (tags: array(t)) => {
      let visibleTags =
        if (state.showingAll || Array.length(tags) <= 25) {
          tags;
        } else {
          selectTopTags(reduxState, tags);
        };
      Array.mapi(
        (index, tag) => {
          let isSelected = Belt.Set.String.has(reduxState, tag##label);
          let className = isSelected ? "tag is-dark" : "tag is-light";
          <a
            key={string_of_int(index)}
            className
            href="#"
            title={string_of_int(tag##count)}
            onClick={_ => reduxDispatch(Redux.ToggleTag(tag##label))}>
            {ReasonReact.string(tag##label)}
          </a>;
        },
        visibleTags,
      );
    };
    <GetTagsQuery>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading tags...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          <div className="tags">
            <span className="tag is-info">
              {ReasonReact.string("Tags")}
            </span>
            {ReasonReact.array(buildTags(response##tags))}
            {allTagsToggle(response##tags)}
          </div>
        }
      }
    </GetTagsQuery>;
  };
};