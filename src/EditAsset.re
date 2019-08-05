//
// Copyright (c) 2018 Nathan Fiedler
//
/* The expected shape of the asset data from GraphQL. */
type t = {
  .
  "id": string,
  "caption": option(string),
  "datetime": Js.Json.t,
  "duration": option(float),
  "filename": string,
  "filepath": string,
  "filesize": Js.Json.t,
  "location": option(string),
  "mimetype": string,
  "tags": Js.Array.t(string),
  "userdate": option(Js.Json.t),
  "previewUrl": string,
  "assetUrl": string,
};

module FetchAsset = [%graphql
  {|
    query Fetch($identifier: ID!) {
      asset(id: $identifier) {
        id
        caption
        datetime
        duration
        filename
        filepath
        filesize
        location
        mimetype
        tags
        userdate
        previewUrl
        assetUrl
      }
    }
  |}
];

module FetchAssetQuery = ReasonApollo.CreateQuery(FetchAsset);

/* The type AssetInput for the mutation. */
type input = {
  .
  "tags": option(array(string)),
  "caption": option(string),
  "location": option(string),
  "datetime": option(Js.Json.t),
  "mimetype": option(string),
};

/*
 * Have the response include all of the fields that the user can modify,
 * that way the Apollo Client will automatically update the cached values.
 */
module UpdateAsset = [%graphql
  {|
    mutation Update($identifier: ID!, $input: AssetInput!) {
      update(id: $identifier, asset: $input) {
        id
        caption
        datetime
        location
        mimetype
        tags
        userdate
      }
    }
  |}
];

module UpdateAssetMutation = ReasonApollo.CreateMutation(UpdateAsset);

module EditForm = {
  open Formality;

  type field =
    | Tags
    | Location
    | Caption
    | UserDate
    | Mimetype;

  type state = {
    tags: string,
    location: string,
    caption: string,
    userdate: string,
    mimetype: string,
  };

  type message = string;
  type submissionError = unit;
  // define this updater type for convenience
  type updater = (state, string) => state;

  let dateRegex = [%bs.re "/^\\d{1,4}-\\d{1,2}-\\d{1,2} \\d{1,2}:\\d{2}$/"];

  module TagsField = {
    let update = (state, value) => {...state, tags: value};

    let validator = {
      field: Tags,
      strategy: Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  module LocationField = {
    let update = (state, value) => {...state, location: value};

    let validator = {
      field: Location,
      strategy: Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  module CaptionField = {
    let update = (state, value) => {...state, caption: value};

    let validator = {
      field: Caption,
      strategy: Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  module UserDateField = {
    let update = (state, value) => {...state, userdate: value};

    let validator = {
      field: UserDate,
      strategy: Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: state =>
        if (String.length(state.userdate) == 0) {
          Ok(Valid);
        } else {
          switch (Js.Re.exec_(dateRegex, state.userdate)) {
          | None => Error("date format must be yyyy-MM-dd HH:mm")
          | Some(_result) => Ok(Valid)
          };
        },
    };
  };

  module MimetypeField = {
    let update = (state, value) => {...state, mimetype: value};

    let validator = {
      field: Mimetype,
      strategy: Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  let validators = [
    TagsField.validator,
    LocationField.validator,
    CaptionField.validator,
    UserDateField.validator,
    MimetypeField.validator,
  ];
};

module EditFormHook = Formality.Make(EditForm);

let assetMimeType = (mimetype: string) =>
  if (mimetype == "video/quicktime") {
    "video/mp4";
  } else {
    mimetype;
  };

let formatDateForDisplay = (datetime: Js.Json.t) =>
  switch (Js.Json.decodeNumber(datetime)) {
  | None => "INVALID DATE"
  | Some(num) =>
    let d = Js.Date.fromFloat(num);
    Js.Date.toLocaleString(d);
  };

let formatDateForInput = (datetime: Js.Json.t) =>
  switch (Js.Json.decodeNumber(datetime)) {
  | None => "INVALID DATE"
  | Some(num) =>
    let d = Js.Date.fromFloat(num);
    DateFns.format("YYYY-MM-DD HH:mm", d);
  };

let formatUserDate = (userdate: option(Js.Json.t)) =>
  switch (userdate) {
  | None => ""
  | Some(datetime) => formatDateForInput(datetime)
  };

let assetPreview = (asset: t) =>
  if (Js.String.startsWith("video/", asset##mimetype)) {
    <video
      style={ReactDOMRe.Style.make(~width="100%", ~height="100%", ())}
      controls=true
      preload="auto">
      <source src=asset##assetUrl type_={assetMimeType(asset##mimetype)} />
      {ReasonReact.string("Bummer, your browser does not support the HTML5")}
      <code> {ReasonReact.string("video")} </code>
      {ReasonReact.string("tag.")}
    </video>;
  } else {
    <a href={"/asset/" ++ asset##id}>
      <figure className="image">
        <img
          style={ReactDOMRe.Style.make(~display="inline", ~width="auto", ())}
          src=asset##previewUrl
          alt=asset##filename
        />
      </figure>
    </a>;
  };

let editFormInput =
    (
      form: EditFormHook.interface,
      field: EditForm.field,
      updater: EditForm.updater,
      labelText: string,
      inputId: string,
      inputType: string,
      inputValue: string,
      placeholderText: string,
    ) => {
  let validateMsg =
    switch (form.result(field)) {
    | Some(Error(message)) => message
    | Some(Ok(Valid | NoValue))
    | None => ""
    };
  let formIsValid = validateMsg == "";
  let inputClass = formIsValid ? "input" : "input is-danger";
  let validationTextDiv =
    <p className="help is-danger"> {ReasonReact.string(validateMsg)} </p>;
  let inputField =
    <div className="control">
      <input
        id=inputId
        className=inputClass
        type_=inputType
        name=inputId
        value=inputValue
        onBlur={_ => form.blur(field)}
        onChange={event =>
          form.change(
            field,
            updater(form.state, event->ReactEvent.Form.target##value),
          )
        }
        placeholder=placeholderText
      />
    </div>;
  let formGroupElems =
    if (formIsValid) {
      inputField;
    } else {
      ReasonReact.array([|inputField, validationTextDiv|]);
    };
  <div className="field is-horizontal">
    <div className="field-label is-normal">
      <label htmlFor=inputId className="label">
        {ReasonReact.string(labelText)}
      </label>
    </div>
    <div className="field-body">
      <div className="field"> formGroupElems </div>
    </div>
  </div>;
};

let assetSaveButton = (form: EditFormHook.interface) =>
  switch (form.status) {
  | Submitting(_) => <p> {React.string("Saving...")} </p>
  | SubmissionFailed(_) =>
    <input type_="submit" value="Save" className="button" disabled=true />
  | _ => <input type_="submit" value="Save" className="button is-primary" />
  };

module EditFormRe = {
  [@react.component]
  let make = (~asset: t, ~onSubmit) => {
    let initial: EditForm.state = {
      tags: Js.Array.joinWith(", ", asset##tags),
      location: Belt.Option.getWithDefault(asset##location, ""),
      caption: Belt.Option.getWithDefault(asset##caption, ""),
      userdate: formatUserDate(asset##userdate),
      mimetype: asset##mimetype,
    };
    let form: EditFormHook.interface =
      EditFormHook.useForm(~initialState=initial, ~onSubmit=(state, _form) =>
        onSubmit(state)
      );
    <form onSubmit={form.submit->Formality.Dom.preventDefault}>
      <div
        className="container"
        style={ReactDOMRe.Style.make(~width="auto", ~paddingRight="3em", ())}>
        {editFormInput(
           form,
           UserDate,
           EditForm.UserDateField.update,
           "Custom Date",
           "userdate",
           "text",
           form.state.userdate,
           "yyyy-mm-dd HH:MM",
         )}
        {editFormInput(
           form,
           Location,
           EditForm.LocationField.update,
           "Location",
           "location",
           "text",
           form.state.location,
           "",
         )}
        {editFormInput(
           form,
           Caption,
           EditForm.CaptionField.update,
           "Caption",
           "caption",
           "text",
           form.state.caption,
           "",
         )}
        {editFormInput(
           form,
           Tags,
           EditForm.TagsField.update,
           "Tags",
           "tags",
           "text",
           form.state.tags,
           "comma-separated values",
         )}
        {editFormInput(
           form,
           Mimetype,
           EditForm.MimetypeField.update,
           "Media type",
           "mimetype",
           "text",
           form.state.mimetype,
           "image/jpeg",
         )}
        <div className="field is-horizontal">
          <div className="field-label" />
          <div className="field-body"> {assetSaveButton(form)} </div>
        </div>
      </div>
    </form>;
  };
};

/* Convert the user date string into UTC milliseconds. */
let userDateStrToInt = str =>
  if (String.length(str) > 0) {
    /* date-fns 1.x parse does not take a format string... */
    let date: Js.Date.t = DateFns.parseString(str);
    Some(Js.Json.number(Js.Date.valueOf(date)));
  } else {
    None;
  };

/*
 * Split the string on commas, replacing None with empty string.
 */
let splitOnComma = (str: string): array(string) => {
  let parts = Js.String.splitByRe([%bs.re "/,/"], str);
  Array.map(a => Belt.Option.getWithDefault(a, ""), parts);
};

let submitUpdate =
    (
      asset: t,
      mutate: UpdateAssetMutation.apolloMutation,
      values: EditForm.state,
    ) => {
  let splitTags = splitOnComma(values.tags);
  /* this may introduce a single blank tag, but it's easier to let the backend prune it */
  let trimmedTags = Array.map(s => String.trim(s), splitTags);
  let newAsset: input = {
    "tags": Some(trimmedTags),
    "caption": Some(values.caption),
    "location": Some(values.location),
    "datetime": userDateStrToInt(values.userdate),
    "mimetype": Some(values.mimetype),
  };
  let update = UpdateAsset.make(~identifier=asset##id, ~input=newAsset, ());
  /* ignore the returned promise, the result will be delivered later */
  mutate(
    ~variables=update##variables,
    ~refetchQueries=[|"getAllTags", "getAllLocations", "getAllYears"|],
    (),
  )
  |> ignore;
};

module EditPanel = {
  [@react.component]
  let make = (~asset: t) => {
    <UpdateAssetMutation>
      ...{(mutate, {result}) =>
        switch (result) {
        | Loading => <p> {ReasonReact.string("Loading...")} </p>
        | Error(error) =>
          Js.log(error);
          <div> {ReasonReact.string(error##message)} </div>;
        | Data(_result) =>
          <div>
            {
              ReasonReact.Router.push("/assets/" ++ asset##id);
              ReasonReact.string("loading...");
            }
          </div>
        | NotCalled =>
          <div className="container">
            <div className="card">
              <div className="card-header">
                <p className="card-header-title">
                  {ReasonReact.string(
                     formatDateForDisplay(asset##datetime)
                     ++ ", "
                     ++
                     asset##filename,
                   )}
                </p>
              </div>
              <div className="card-image has-text-centered">
                {assetPreview(asset)}
              </div>
              <div className="card-content">
                <EditFormRe asset onSubmit={submitUpdate(asset, mutate)} />
              </div>
            </div>
          </div>
        }
      }
    </UpdateAssetMutation>;
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
          switch (response##asset) {
          | None => <div> {ReasonReact.string("No such asset!")} </div>
          | Some(asset) => <EditPanel asset />
          }
        }
      }
    </FetchAssetQuery>;
  };
};