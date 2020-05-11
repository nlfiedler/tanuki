//
// Copyright (c) 2020 Nathan Fiedler
//

/*
 * The search page presents a form in place of the attribute selectors, and
 * displays the results as a grid of thumbnails.
 */
module SearchAssets = [%graphql
  {|
    query Search($params: SearchParams!, $pageSize: Int, $offset: Int) {
      search(params: $params, count: $pageSize, offset: $offset) {
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

module SearchAssetsQuery = ReasonApollo.CreateQuery(SearchAssets);

module SearchForm = {
  // open Formality;

  type field =
    | Tags
    | Locations
    | AfterDate
    | BeforeDate
    | Filename
    | Mimetype;

  type state = Redux.searchInputs;

  type message = string;
  type submissionError = unit;
  // define this updater type for convenience
  type updater = (state, string) => state;

  module TagsField = {
    let update = (state: state, value) => {...state, tags: value};

    let validator = {
      Formality.field: Tags,
      strategy: Formality.Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  module LocationsField = {
    let update = (state: state, value) => {...state, locations: value};

    let validator = {
      Formality.field: Locations,
      strategy: Formality.Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  let dateRegex = [%bs.re "/^\\d{1,4}-\\d{1,2}-\\d{1,2}$/"];

  let dateValidator = (value: string): Formality.result(string) =>
    if (String.length(value) == 0) {
      Ok(Valid);
    } else {
      switch (Js.Re.exec_(dateRegex, value)) {
      | None => Error("date format must be yyyy-MM-dd HH:mm")
      | Some(_result) => Ok(Valid)
      };
    };

  module AfterDateField = {
    let update = (state: state, value) => {...state, afterDate: value};

    let validator = {
      Formality.field: AfterDate,
      strategy: Formality.Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: (state: state) => dateValidator(state.afterDate),
    };
  };

  module BeforeDateField = {
    let update = (state: state, value) => {...state, beforeDate: value};

    let validator = {
      Formality.field: BeforeDate,
      strategy: Formality.Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: (state: state) => dateValidator(state.beforeDate),
    };
  };

  module FilenameField = {
    let update = (state: state, value) => {...state, filename: value};

    let validator = {
      Formality.field: Filename,
      strategy: Formality.Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  module MimetypeField = {
    let update = (state: state, value) => {...state, mimetype: value};

    let validator = {
      Formality.field: Mimetype,
      strategy: Formality.Strategy.OnFirstSuccessOrFirstBlur,
      dependents: None,
      validate: _state => Ok(Valid),
    };
  };

  let validators = [
    TagsField.validator,
    LocationsField.validator,
    AfterDateField.validator,
    BeforeDateField.validator,
    FilenameField.validator,
    MimetypeField.validator,
  ];
};

module SearchFormHook = Formality.Make(SearchForm);

let searchFormInput =
    (
      form: SearchFormHook.interface,
      field: SearchForm.field,
      updater: SearchForm.updater,
      labelText: string,
      inputId: string,
      inputType: string,
      inputValue: string,
      placeholderText: string,
      iconClass: string,
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
    <p className="control is-expanded has-icons-left">
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
      <span className="icon is-small is-left">
        <i className=iconClass />
      </span>
    </p>;
  let field =
    if (formIsValid) {
      inputField;
    } else {
      ReasonReact.array([|inputField, validationTextDiv|]);
    };
  <>
    <div className="field-label is-normal">
      <label className="label"> {ReasonReact.string(labelText)} </label>
    </div>
    <div className="field"> field </div>
  </>;
};

/* Construct the search form, populating it with the saved values, if any. */
module SearchFormRe = {
  [@react.component]
  let make = (~inputs, ~onSubmit) => {
    let form: SearchFormHook.interface =
      SearchFormHook.useForm(
        ~initialState=inputs,
        ~onSubmit=(state, form) => {
          onSubmit(state);
          // reset the form so it will accept input and submit properly
          // after the search results are rendered (i.e. search for one
          // thing and then try another search, nothing would happen)
          form.reset();
        },
      );
    <form onSubmit={form.submit->Formality.Dom.preventDefault}>
      <div
        className="container"
        style={ReactDOMRe.Style.make(
          ~width="auto",
          ~paddingRight="3em",
          ~marginBottom="1em",
          (),
        )}>
        <div className="field is-horizontal">
          <div className="field-body">
            {searchFormInput(
               form,
               Tags,
               SearchForm.TagsField.update,
               "Tags",
               "tags",
               "text",
               form.state.tags,
               "comma-separated values",
               "fas fa-tags",
             )}
            {searchFormInput(
               form,
               Locations,
               SearchForm.LocationsField.update,
               "Locations",
               "locations",
               "text",
               form.state.locations,
               "comma-separated values",
               "fas fa-map",
             )}
          </div>
        </div>
        <div className="field is-horizontal">
          <div className="field-body">
            {searchFormInput(
               form,
               AfterDate,
               SearchForm.AfterDateField.update,
               "After date",
               "after",
               "text",
               form.state.afterDate,
               "2002-01-31",
               "fas fa-calendar",
             )}
            {searchFormInput(
               form,
               BeforeDate,
               SearchForm.BeforeDateField.update,
               "Before date",
               "before",
               "text",
               form.state.beforeDate,
               "2003-08-30",
               "fas fa-calendar",
             )}
          </div>
        </div>
        <div className="field is-horizontal">
          <div className="field-body">
            {searchFormInput(
               form,
               Filename,
               SearchForm.FilenameField.update,
               "Filename",
               "filename",
               "text",
               form.state.filename,
               "img_1234.jpg",
               "fas fa-file",
             )}
            {searchFormInput(
               form,
               Mimetype,
               SearchForm.MimetypeField.update,
               "Media type",
               "mimetype",
               "text",
               form.state.mimetype,
               "image/jpeg",
               "fas fa-code",
             )}
          </div>
        </div>
        <div className="field is-grouped is-grouped-right">
          <div className="control">
            <input
              type_="submit"
              value="Search"
              className="button is-primary"
            />
          </div>
        </div>
      </div>
    </form>;
  };
};

/* Convert the yyyy-MM-dd date string into what GraphQL server expects. */
let userDateToGraphQL = str =>
  if (String.length(str) > 0) {
    Some(Js.Json.string(str ++ "T00:00:00Z"));
  } else {
    None;
  };

/*
 * Split the string on commas, discarding empty strings.
 */
let stringToArray = (str: string): array(string) => {
  let parts = Js.String.split(",", str);
  let trimmed = Array.map(s => String.trim(s), parts);
  Js.Array.filter(e => String.length(e) > 0, trimmed);
};

/*
 * Convert the form parameters into GraphQL search parameters.
 */
let makeSearchParams = (params: SearchForm.state) => {
  let nonEmptyTags = stringToArray(params.tags);
  let tags = Array.length(nonEmptyTags) > 0 ? Some(nonEmptyTags) : None;
  let nonEmptyLocations = stringToArray(params.locations);
  let locations =
    Array.length(nonEmptyLocations) > 0 ? Some(nonEmptyLocations) : None;
  {
    "after": userDateToGraphQL(params.afterDate),
    "before": userDateToGraphQL(params.beforeDate),
    "filename":
      String.length(params.filename) > 0 ? Some(params.filename) : None,
    "locations": locations,
    "mimetype":
      String.length(params.mimetype) > 0 ? Some(params.mimetype) : None,
    "tags": tags,
  };
};

// React hooks require a stable function reference to work properly.
let stateSelector = (state: Redux.appState) => state;

module Component = {
  type state = {params: SearchForm.state};
  type action =
    | SetParams(SearchForm.state);
  [@react.component]
  let make = () => {
    let reduxState = Redux.useSelector(stateSelector);
    let reduxDispatch = Redux.useDispatch();
    let (state, dispatch) =
      React.useReducer(
        (_state, action) =>
          switch (action) {
          | SetParams(params) => {params: params}
          },
        {params: reduxState.savedSearch},
      );
    let onSubmit = params => {
      reduxDispatch(Redux.Paginate(1));
      reduxDispatch(Redux.SaveSearch(params));
      dispatch(SetParams(params));
    };
    <div>
      <SearchFormRe inputs={reduxState.savedSearch} onSubmit />
      {
        let offset = (reduxState.pageNumber - 1) * Thumbnails.pageSize;
        let queryParams = makeSearchParams(state.params);
        let query =
          SearchAssets.make(
            ~params=queryParams,
            ~pageSize=Thumbnails.pageSize,
            ~offset,
            (),
          );
        <SearchAssetsQuery variables=query##variables>
          ...{({result}) =>
            switch (result) {
            | Loading => <div> {ReasonReact.string("Loading...")} </div>
            | Error(error) =>
              Js.log(error);
              <div> {ReasonReact.string(error##message)} </div>;
            | Data(response) =>
              <Thumbnails.Component
                state=reduxState
                dispatch=reduxDispatch
                search=response##search
              />
            }
          }
        </SearchAssetsQuery>;
      }
    </div>;
  };
};