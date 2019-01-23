/* The expected shape of the asset data from GraphQL. */
type t = {
  .
  "id": string,
  "caption": option(string),
  "datetime": Js.Json.t,
  "duration": option(float),
  "filename": string,
  "filepath": string,
  "filesize": int,
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

module EditFormParams = {
  type state = {
    tags: string,
    location: string,
    caption: string,
    userdate: string,
    mimetype: string,
  };
  type fields = [ | `tags | `location | `caption | `userdate | `mimetype];
  /* lens: [(fieldName, getter, setter)] */
  let lens = [
    (`tags, s => s.tags, (s, tags) => {...s, tags}),
    (`location, s => s.location, (s, location) => {...s, location}),
    (`caption, s => s.caption, (s, caption) => {...s, caption}),
    (`userdate, s => s.userdate, (s, userdate) => {...s, userdate}),
    (`mimetype, s => s.mimetype, (s, mimetype) => {...s, mimetype}),
  ];
};

module EditForm = ReForm.Create(EditFormParams);

let dateRegex = [%bs.re "/^\\d{1,4}-\\d{1,2}-\\d{1,2} \\d{1,2}:\\d{2}$/"];

let dateValidator: string => option(string) =
  value =>
    if (String.length(value) == 0) {
      None;
    } else {
      switch (Js.Re.exec(value, dateRegex)) {
      | None => Some("date format must be yyyy-MM-dd HH:mm")
      | Some(_result) => None
      };
    };

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
      handleChange,
      getErrorForField,
      fieldName,
      labelText,
      inputId,
      inputType,
      inputValue,
      placeholderText,
    ) => {
  let validateMsg =
    Belt.Option.getWithDefault(getErrorForField(fieldName), "");
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
        onChange={
          ReForm.Helpers.handleDomFormChange(handleChange(fieldName))
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

let assetSaveButton = (form: EditForm.state) =>
  switch (form.error) {
  | None => <input type_="submit" value="Save" className="button is-primary" />
  | Some(_error) =>
    <input type_="submit" value="Save" className="button" disabled=true />
  };

module EditFormRe = {
  let component = ReasonReact.statelessComponent("EditForm");
  let make = (~asset: t, ~onSubmit, _children) => {
    ...component,
    render: _self =>
      <EditForm
        onSubmit={({values}) => onSubmit(values)}
        initialState={
          tags: Js.Array.joinWith(", ", asset##tags),
          location: Belt.Option.getWithDefault(asset##location, ""),
          caption: Belt.Option.getWithDefault(asset##caption, ""),
          userdate: formatUserDate(asset##userdate),
          mimetype: asset##mimetype,
        }
        schema=[
          (
            `userdate,
            ReForm.Validation.Custom(
              values => dateValidator(values.userdate),
            ),
          ),
        ]>
        ...{
             ({handleSubmit, handleChange, form, getErrorForField}) =>
               <form
                 onSubmit={ReForm.Helpers.handleDomFormSubmit(handleSubmit)}>
                 <div
                   className="container"
                   style={
                     ReactDOMRe.Style.make(
                       ~width="auto",
                       ~paddingRight="3em",
                       (),
                     )
                   }>
                   {
                     editFormInput(
                       handleChange,
                       getErrorForField,
                       `userdate,
                       "Custom Date",
                       "userdate",
                       "text",
                       form.values.userdate,
                       "yyyy-mm-dd HH:MM",
                     )
                   }
                   {
                     editFormInput(
                       handleChange,
                       getErrorForField,
                       `location,
                       "Location",
                       "location",
                       "text",
                       form.values.location,
                       "",
                     )
                   }
                   {
                     editFormInput(
                       handleChange,
                       getErrorForField,
                       `caption,
                       "Caption",
                       "caption",
                       "text",
                       form.values.caption,
                       "",
                     )
                   }
                   {
                     editFormInput(
                       handleChange,
                       getErrorForField,
                       `tags,
                       "Tags",
                       "tags",
                       "text",
                       form.values.tags,
                       "comma-separated values",
                     )
                   }
                   {
                     editFormInput(
                       handleChange,
                       getErrorForField,
                       `mimetype,
                       "Media type",
                       "mimetype",
                       "text",
                       form.values.mimetype,
                       "image/jpeg",
                     )
                   }
                   <div className="field is-horizontal">
                     <div className="field-label" />
                     <div className="field-body">
                       {assetSaveButton(form)}
                     </div>
                   </div>
                 </div>
               </form>
           }
      </EditForm>,
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

let submitUpdate =
    (
      asset: t,
      mutate: UpdateAssetMutation.apolloMutation,
      values: EditFormParams.state,
    ) => {
  let splitTags = Js.String.splitByRe([%bs.re "/,/"], values.tags);
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
  let component = ReasonReact.statelessComponent("EditAsset");
  let make = (~asset: t, _children) => {
    ...component,
    render: _self =>
      <UpdateAssetMutation>
        ...{
             (mutate, {result}) =>
               switch (result) {
               | Loading => <p> {ReasonReact.string("Loading...")} </p>
               | Error(error) =>
                 Js.log(error);
                 <div> {ReasonReact.string(error##message)} </div>;
               | Data(_result) =>
                 <div>
                  {ReasonReact.Router.push("/assets/" ++ asset##id);
                   ReasonReact.string("loading...")}
                 </div>
               | NotCalled =>
                 <div className="container">
                   <div className="card">
                     <div className="card-header">
                       <p className="card-header-title">
                         {
                           ReasonReact.string(
                             formatDateForDisplay(asset##datetime)
                             ++ ", "
                             ++
                             asset##filename,
                           )
                         }
                       </p>
                     </div>
                     <div className="card-image has-text-centered">
                       {assetPreview(asset)}
                     </div>
                     <div className="card-content">
                       <EditFormRe
                         asset
                         onSubmit={submitUpdate(asset, mutate)}
                       />
                     </div>
                   </div>
                 </div>
               }
           }
      </UpdateAssetMutation>,
  };
};

module Component = {
  let component = ReasonReact.statelessComponent("EditAsset");
  let make = (~assetId: string, _children) => {
    ...component,
    render: _self => {
      let query = FetchAsset.make(~identifier=assetId, ());
      <FetchAssetQuery variables=query##variables>
        ...{
             ({result}) =>
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
    },
  };
};