//
// Copyright (c) 2020 Nathan Fiedler
//
/* The expected shape of the asset data from GraphQL. */
type t = {
  .
  "id": string,
  "caption": option(string),
  "datetime": Js.Json.t,
  "duration": option(int),
  "filename": string,
  "filesize": Js.Json.t,
  "location": option(string),
  "mimetype": string,
  "tags": Js.Array.t(string),
  "userdate": option(Js.Json.t),
};

module FetchAsset = [%graphql
  {|
    query Fetch($identifier: String!) {
      asset(id: $identifier) {
        id
        caption
        datetime
        duration
        filename
        filesize
        location
        mimetype
        tags
        userdate
      }
    }
  |}
];

module FetchAssetQuery = ReasonApollo.CreateQuery(FetchAsset);

let assetMimeType = (mimetype: string) =>
  if (mimetype == "video/quicktime") {
    "video/mp4";
  } else {
    mimetype;
  };

let formatDate = (datetime: Js.Json.t) =>
  switch (Js.Json.decodeString(datetime)) {
  | None => "INVALID STRING"
  | Some(dateStr) =>
    let d = Js.Date.fromString(dateStr);
    Js.Date.toLocaleString(d);
  };

let formatBigInt = (bigint: Js.Json.t) =>
  switch (Js.Json.decodeString(bigint)) {
  | None => "INVALID STRING"
  | Some(num) => num
  };

let assetPreview = (asset: t) =>
  if (Js.String.startsWith("video/", asset##mimetype)) {
    <video
      style={ReactDOMRe.Style.make(~width="100%", ~height="100%", ())}
      controls=true
      preload="auto">
      <source
        src={"/asset/" ++ asset##id}
        type_={assetMimeType(asset##mimetype)}
      />
      {ReasonReact.string("Bummer, your browser does not support the HTML5")}
      <code> {ReasonReact.string("video")} </code>
      {ReasonReact.string("tag.")}
    </video>;
  } else {
    <a href={"/asset/" ++ asset##id}>
      <figure className="image">
        <img
          style={ReactDOMRe.Style.make(~display="inline", ~width="auto", ())}
          src={"/thumbnail/640/640/" ++ asset##id}
          alt=asset##filename
        />
      </figure>
    </a>;
  };

let assetDetails = (asset: t) =>
  <table className="table is-striped is-fullwidth">
    <tbody>
      <tr>
        <td> {ReasonReact.string("Date")} </td>
        <td> {ReasonReact.string(formatDate(asset##datetime))} </td>
      </tr>
      <tr>
        <td> {ReasonReact.string("Size")} </td>
        <td> {ReasonReact.string(formatBigInt(asset##filesize))} </td>
      </tr>
      {switch (asset##duration) {
       | None => ReasonReact.null
       | Some(value) =>
         <tr>
           <td> {ReasonReact.string("Duration")} </td>
           <td>
             {ReasonReact.string(string_of_int(value) ++ "seconds")}
           </td>
         </tr>
       }}
      <tr>
        <td> {ReasonReact.string("Location")} </td>
        <td>
          {ReasonReact.string(
             Belt.Option.getWithDefault(asset##location, ""),
           )}
        </td>
      </tr>
      <tr>
        <td> {ReasonReact.string("Caption")} </td>
        <td>
          {ReasonReact.string(Belt.Option.getWithDefault(asset##caption, ""))}
        </td>
      </tr>
      <tr>
        <td> {ReasonReact.string("Tags")} </td>
        <td> {ReasonReact.string(Js.Array.joinWith(", ", asset##tags))} </td>
      </tr>
      <tr>
        <td> {ReasonReact.string("Media type")} </td>
        <td> {ReasonReact.string(asset##mimetype)} </td>
      </tr>
    </tbody>
  </table>;

module PreviewPanel = {
  [@react.component]
  let make = (~asset: t) => {
    <div className="container">
      <div className="card">
        <div className="card-header">
          <p className="card-header-title">
            {ReasonReact.string(asset##filename)}
          </p>
          <a
            className="card-header-icon"
            onClick={_ =>
              ReasonReact.Router.push("/assets/" ++ asset##id ++ "/edit")
            }>
            <span className="icon"> <i className="fa fa-edit" /> </span>
          </a>
        </div>
        <div className="card-image has-text-centered">
          {assetPreview(asset)}
        </div>
        <div className="card-content"> {assetDetails(asset)} </div>
      </div>
    </div>;
  };
};

module Component = {
  [@react.component]
  let make = (~assetId: string) => {
    let query = FetchAsset.make(~identifier=assetId, ());
    <FetchAssetQuery variables=query##variables>
      ...{({result}) =>
        switch (result) {
        | Loading => <div> {ReasonReact.string("Loading...")} </div>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(response) =>
          <PreviewPanel
            asset={
              response##asset;
            }
          />
        }
      }
    </FetchAssetQuery>;
  };
};