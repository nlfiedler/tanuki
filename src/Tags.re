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
 *
 * A clever alternative would be to find the elbow/knee of the data but that
 * requires a lot more math than Elm is really suitable for.
 */
let selectTopTags = allTags => allTags;
/* TODO: need the selected tags to be in the reductive model */
/* TODO: just need the tag label, not the whole object */
/* TODO: the selected state is also used for selecting assets */
/* {
       let
            /* Get the selected tags into a dict. */
           selectedTags =
               List.filter (.selected) tags
           mapInserter v d =
               Dict.insert v.label v d
           selectedTagsDict =
               List.foldl mapInserter Dict.empty selectedTags
            /* Get the top N tags by count. */
           tagSorter a b =
               case compare a.count b.count of
                   LT -> GT
                   EQ -> EQ
                   GT -> LT
           sortedTopTags =
               List.take 25 (List.sortWith tagSorter tags)
            /* Merge those two sets into one. */
           mergedTagsDict =
               List.foldl mapInserter selectedTagsDict sortedTopTags
       in
           /* Extract the values and sort by label. */
           List.sortBy .label (Dict.values mergedTagsDict)
   }; */

module Component = {
  type state = {showingAll: bool};
  type action =
    | ToggleAll;
  let component = ReasonReact.reducerComponent("Tags");
  let make = _children => {
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
    let buildTags = (state, tags) => {
      let visibleTags =
        if (state.showingAll || Array.length(tags) <= 25) {
          tags;
        } else {
          selectTopTags(tags);
        };
      /* TODO: use class "tag is-dark" for selected tags */
      Array.mapi(
        (index, tag) =>
          <a
            key={string_of_int(index)}
            className="tag is-light"
            href="#"
            title={string_of_int(tag##count)}
            onClick={_ => ()}>
            {ReasonReact.string(tag##value)}
          </a>,
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