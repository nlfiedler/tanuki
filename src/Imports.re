//
// Copyright (c) 2020 Nathan Fiedler
//

module RecentImports = [%graphql
  {|
    query Recent($since: DateTimeUtc) {
      recent(since: $since) {
        results {
          id
          datetime
          filename
          location
          mimetype
        }
        count
      }
    }
  |}
];

module RecentImportsQuery = ReasonApollo.CreateQuery(RecentImports);

// Type for the search results.
type t = {
  .
  "datetime": Js.Json.t,
  "filename": string,
  "id": string,
  "location": option(string),
  "mimetype": string,
};

/* The input type AssetInputId for the mutation. */
type input = {
  .
  "id": string,
  "input": {
    .
    "caption": option(string),
    "datetime": option(Js.Json.t),
    "location": option(string),
    "mimetype": option(string),
    "tags": option(array(string)),
  },
};

module UpdateAssets = [%graphql
  {|
    mutation BulkUpdate($assets: [AssetInputId!]!) {
      bulkUpdate(assets: $assets)
    }
  |}
];

module UpdateAssetsMutation = ReasonApollo.CreateMutation(UpdateAssets);

type timeRange =
  | Day
  | Week
  | Month
  | AllTime;

let makeDate = (from: timeRange) => {
  let daysAgo = (ago: float) => {
    let base = Js.Date.make();
    let date = DateFns.subDays(ago, base);
    let fmt = DateFns.format("YYYY-MM-DD[T]HH:mm:ssZ", date);
    Js.Json.string(fmt);
  };
  switch (from) {
  | Day => daysAgo(1.)
  | Week => daysAgo(7.)
  | Month => daysAgo(30.)
  | AllTime => Js.Json.null
  };
};

let buttonBar = (count, showing, dispatch) => {
  let makeButton = (range, label) =>
    if (range == showing) {
      <a className="button is-dark"> {ReasonReact.string(label)} </a>;
    } else {
      <a className="button is-light" onClick={_ => dispatch(range)}>
        {ReasonReact.string(label)}
      </a>;
    };
  <nav className="level">
    <div className="level-left">
      <div className="level-item" />
      <p className="subtitle is-5">
        <strong style={ReactDOMRe.Style.make(~paddingRight=".5em", ())}>
          {ReasonReact.string(string_of_int(count))}
        </strong>
        {ReasonReact.string("assets need attention...")}
      </p>
    </div>
    <div className="level-right">
      <p className="level-item"> {makeButton(Day, "Day")} </p>
      <p className="level-item"> {makeButton(Week, "Week")} </p>
      <p className="level-item"> {makeButton(Month, "Month")} </p>
      <p className="level-item"> {makeButton(AllTime, "All Time")} </p>
    </div>
  </nav>;
};

let brokenThumbnailPlaceholder = filename =>
  switch (Mimetypes.filenameToMediaType(filename)) {
  | Mimetypes.Image => "file-picture.png"
  | Mimetypes.Video => "file-video-2.png"
  | Mimetypes.Pdf => "file-acrobat.png"
  | Mimetypes.Audio => "file-music-3.png"
  | Mimetypes.Text => "file-new-2.png"
  | Mimetypes.Unknown => "file-new-1.png"
  };

let assetMimeType = (mimetype: string) =>
  if (mimetype == "video/quicktime") {
    "video/mp4";
  } else {
    mimetype;
  };

module ThumbCard = {
  type state = {thumbless: bool};
  type action =
    | MarkThumbless;
  [@react.component]
  let make = (~entry) => {
    let (state, dispatch) =
      React.useReducer(
        (_state, action) =>
          switch (action) {
          | MarkThumbless => {thumbless: true}
          },
        {thumbless: false},
      );
    <figure className="image">
      {state.thumbless
         ? <img
             src={"/images/" ++ brokenThumbnailPlaceholder(entry##filename)}
             alt=entry##filename
             style={ReactDOMRe.Style.make(~width="auto", ())}
           />
         : (
           if (Js.String.startsWith("video/", entry##mimetype)) {
             <video width="240" controls=true preload="auto">
               <source
                 src={"/api/asset/" ++ entry##id}
                 type_={assetMimeType(entry##mimetype)}
               />
               {ReasonReact.string(
                  "Bummer, your browser does not support the HTML5",
                )}
               <code> {ReasonReact.string("video")} </code>
               {ReasonReact.string("tag.")}
             </video>;
           } else {
             <img
               src={"/api/thumbnail/240/240/" ++ entry##id}
               alt=entry##filename
               onError={_ => dispatch(MarkThumbless)}
               style={ReactDOMRe.Style.make(~width="auto", ())}
             />;
           }
         )}
    </figure>;
  };
};

let valueFromEvent = (evt): string => ReactEvent.Form.target(evt)##value;

module BulkForm = {
  type state = {captions: array(string)};
  type action =
    | SetCaption(int, string);
  [@react.component]
  let make = (~results: array(t), ~onSubmit) => {
    let (state, dispatch) =
      React.useReducer(
        (state, action) =>
          switch (action) {
          | SetCaption(index, value) =>
            // cannot modify the existing state object
            let captions = Array.copy(state.captions);
            captions[index] = value;
            {captions: captions};
          },
        {captions: Array.make(Array.length(results), "")},
      );
    let makeRow = (index: int, entry: t) => {
      <div
        className="columns"
        key={
          entry##id;
        }>
        <div
          className="column is-one-third"
          onClick={_ => ReasonReact.Router.push("/assets/" ++ entry##id)}
          style={ReactDOMRe.Style.make(~cursor="pointer", ())}>
          <ThumbCard entry />
          <small style={ReactDOMRe.Style.make(~wordWrap="break-word", ())}>
            {ReasonReact.string(entry##filename)}
          </small>
        </div>
        <div className="column is-two-thirds">
          <div className="field">
            <div
              className="control is-expanded has-icons-left has-icons-right">
              <input
                id={"input" ++ string_of_int(index)}
                className="input"
                type_="text"
                name={"input" ++ string_of_int(index)}
                value={Array.get(state.captions, index)}
                placeholder="Caption with #tags and @location or @\"some location\""
                onChange={evt =>
                  dispatch(SetCaption(index, valueFromEvent(evt)))
                }
              />
              <span className="icon is-small is-left">
                <i className="fas fa-quote-left" />
              </span>
              <span className="icon is-small is-right">
                <i className="fas fa-quote-right" />
              </span>
              <p className="help">
                {ReasonReact.string(
                   "Enter a description, including #tags and @location",
                 )}
              </p>
            </div>
          </div>
        </div>
      </div>;
    };
    <form
      onSubmit={evt => {
        evt->ReactEvent.Synthetic.preventDefault;
        onSubmit(state.captions);
      }}>
      <div className="notification has-text-centered">
        {ReasonReact.string(
           "Fill in some or all of the fields and click the Save button below.",
         )}
      </div>
      {ReasonReact.array(
         {Array.mapi(
            (index: int, entry: t) => makeRow(index, entry),
            results,
          )},
       )}
      <div className="field is-grouped is-grouped-centered">
        <div className="control">
          <input
            type_="submit"
            value="Save All Changes"
            className="button is-primary"
          />
        </div>
      </div>
    </form>;
  };
};

let submitUpdate =
    (
      assets: array(t),
      mutate: UpdateAssetsMutation.apolloMutation,
      values: array(string),
    ) => {
  let pairs = Belt.Array.zip(assets, values);
  let full_pairs =
    Js.Array.filter(((_, v)) => String.length(v) > 0, pairs);
  let assets: array(input) =
    Array.map(
      ((asset, value)) =>
        {
          "id": asset##id,
          "input": {
            "tags": None,
            "caption": Some(value),
            "location": None,
            "datetime": None,
            "mimetype": None,
          },
        },
      full_pairs,
    );
  let update = UpdateAssets.make(~assets, ());
  /* ignore the returned promise, the result will be delivered later */
  mutate(
    ~variables=update##variables,
    ~refetchQueries=[|"getAllTags", "getAllLocations", "getAllYears"|],
    (),
  )
  |> ignore;
};

module BulkUpdate = {
  [@react.component]
  let make = (~results: array(t)) => {
    <UpdateAssetsMutation>
      ...{(mutate, {result}) =>
        switch (result) {
        | Loading => <p> {ReasonReact.string("Loading...")} </p>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(_result) =>
          <div className="container">
            <div className="notification has-text-centered">
              {ReasonReact.string("Changes submitted")}
            </div>
          </div>
        | NotCalled =>
          if (Array.length(results) > 0) {
            <div style={ReactDOMRe.Style.make(~marginBottom="3em", ())}>
              <BulkForm results onSubmit={submitUpdate(results, mutate)} />
            </div>;
          } else {
            <div className="container">
              <div className="notification has-text-centered">
                {ReasonReact.string(
                   "Use the time period selectors above to find pending assets.",
                 )}
              </div>
            </div>;
          }
        }
      }
    </UpdateAssetsMutation>;
  };
};

module Component = {
  type state = {showing: timeRange};
  type action =
    | SetShowing(timeRange);
  [@react.component]
  let make = () => {
    let (state, dispatch) =
      React.useReducer(
        (_state, action) =>
          switch (action) {
          | SetShowing(value) => {showing: value}
          },
        {showing: Day},
      );
    let since = makeDate(state.showing);
    let query = RecentImports.make(~since, ());
    let setShowing = range => dispatch(SetShowing(range));
    <RecentImportsQuery variables=query##variables>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          <div>
            {buttonBar(response##recent##count, state.showing, setShowing)}
            <BulkUpdate results=response##recent##results />
          </div>
        }
      }
    </RecentImportsQuery>;
  };
};