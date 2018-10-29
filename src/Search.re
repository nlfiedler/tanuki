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
        }
        count
      }
    }
  |}
];

module SearchAssetsQuery = ReasonApollo.CreateQuery(SearchAssets);

module SearchFormParams = {
  type state = Redux.searchInputs;
  type fields = [
    | `tags
    | `locations
    | `afterDate
    | `beforeDate
    | `filename
    | `mimetype
  ];
  /* lens: [(fieldName, getter, setter)] */
  let lens = [
    /* the state type alias above requires clearly annotating these functions */
    (
      `tags,
      (s: state) => (s.tags: string),
      (s: state, tags) => {...s, tags},
    ),
    (`locations, s => s.locations, (s, locations) => {...s, locations}),
    (`afterDate, s => s.afterDate, (s, afterDate) => {...s, afterDate}),
    (`beforeDate, s => s.beforeDate, (s, beforeDate) => {...s, beforeDate}),
    (`filename, s => s.filename, (s, filename) => {...s, filename}),
    (`mimetype, s => s.mimetype, (s, mimetype) => {...s, mimetype}),
  ];
};

module SearchForm = ReForm.Create(SearchFormParams);

let dateRegex = [%bs.re "/^\\d{1,4}-\\d{1,2}-\\d{1,2}$/"];

let dateValidator: string => option(string) =
  value =>
    if (String.length(value) == 0) {
      None;
    } else {
      switch (Js.Re.exec(value, dateRegex)) {
      | None => Some("date format must be yyyy-MM-dd")
      | Some(_result) => None
      };
    };

let searchFormInput =
    (
      handleChange,
      getErrorForField,
      fieldName,
      labelText,
      inputId,
      inputType,
      inputValue,
      placeholderText,
      iconClass,
    ) => {
  let validateMsg =
    Belt.Option.getWithDefault(getErrorForField(fieldName), "");
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
        onChange={
          ReForm.Helpers.handleDomFormChange(handleChange(fieldName))
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
  let component = ReasonReact.statelessComponent("SearchForm");
  let make = (~inputs, ~onSubmit, _children) => {
    ...component,
    render: _self =>
      <SearchForm
        onSubmit={({values}) => onSubmit(values)}
        initialState=inputs
        schema=[
          (
            `afterDate,
            ReForm.Validation.Custom(
              values => dateValidator(values.afterDate),
            ),
          ),
          (
            `beforeDate,
            ReForm.Validation.Custom(
              values => dateValidator(values.beforeDate),
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
                       ~marginBottom="1em",
                       (),
                     )
                   }>
                   <div className="field is-horizontal">
                     <div className="field-body">
                       {
                         searchFormInput(
                           handleChange,
                           getErrorForField,
                           `tags,
                           "Tags",
                           "tags",
                           "text",
                           form.values.tags,
                           "comma-separated values",
                           "fas fa-tags",
                         )
                       }
                       {
                         searchFormInput(
                           handleChange,
                           getErrorForField,
                           `locations,
                           "Locations",
                           "locations",
                           "text",
                           form.values.locations,
                           "comma-separated values",
                           "fas fa-map",
                         )
                       }
                     </div>
                   </div>
                   <div className="field is-horizontal">
                     <div className="field-body">
                       {
                         searchFormInput(
                           handleChange,
                           getErrorForField,
                           `afterDate,
                           "After date",
                           "after",
                           "text",
                           form.values.afterDate,
                           "2002-01-31",
                           "fas fa-calendar",
                         )
                       }
                       {
                         searchFormInput(
                           handleChange,
                           getErrorForField,
                           `beforeDate,
                           "Before date",
                           "before",
                           "text",
                           form.values.beforeDate,
                           "2003-08-30",
                           "fas fa-calendar",
                         )
                       }
                     </div>
                   </div>
                   <div className="field is-horizontal">
                     <div className="field-body">
                       {
                         searchFormInput(
                           handleChange,
                           getErrorForField,
                           `filename,
                           "Filename",
                           "filename",
                           "text",
                           form.values.filename,
                           "img_1234.jpg",
                           "fas fa-file",
                         )
                       }
                       {
                         searchFormInput(
                           handleChange,
                           getErrorForField,
                           `mimetype,
                           "Media type",
                           "mimetype",
                           "text",
                           form.values.mimetype,
                           "image/jpeg",
                           "fas fa-code",
                         )
                       }
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
               </form>
           }
      </SearchForm>,
  };
};

/* Convert the yyyy-MM-dd date string into UTC milliseconds. */
let rangeDateStrToInt = str =>
  if (String.length(str) > 0) {
    /* date-fns 1.x parse does not take a format string... */
    let date: Js.Date.t = DateFns.parseString(str);
    Some(Js.Json.number(Js.Date.valueOf(date)));
  } else {
    None;
  };

/*
 * Convert the form parameters into GraphQL search parameters.
 */
let makeSearchParams = (params: SearchFormParams.state) => {
  let filterEmpties = lst =>
    Array.fold_right(
      (s, acc) => Js.String.length(s) > 0 ? [s, ...acc] : acc,
      lst,
      [],
    );
  let splitTags = Js.String.splitByRe([%bs.re "/,/"], params.tags);
  let trimmedTags = Array.map(s => String.trim(s), splitTags);
  let nonEmptyTags = filterEmpties(trimmedTags);
  let tags =
    List.length(nonEmptyTags) > 0 ?
      Some(Array.of_list(nonEmptyTags)) : None;
  let splitLocations = Js.String.splitByRe([%bs.re "/,/"], params.locations);
  let trimmedLocations = Array.map(s => String.trim(s), splitLocations);
  let nonEmptyLocations = filterEmpties(trimmedLocations);
  let locations =
    List.length(nonEmptyLocations) > 0 ?
      Some(Array.of_list(nonEmptyLocations)) : None;
  {
    "after": rangeDateStrToInt(params.afterDate),
    "before": rangeDateStrToInt(params.beforeDate),
    "filename":
      String.length(params.filename) > 0 ? Some(params.filename) : None,
    "locations": locations,
    "mimetype":
      String.length(params.mimetype) > 0 ? Some(params.mimetype) : None,
    "tags": tags,
  };
};

module SearchRe = {
  type state = {params: SearchFormParams.state};
  type action =
    | SetParams(SearchFormParams.state);
  let component = ReasonReact.reducerComponent("SearchRe");
  let make = (~state: Redux.appState, ~dispatch, _children) => {
    ...component,
    initialState: () => {params: state.savedSearch},
    reducer: action =>
      switch (action) {
      | SetParams(params) => (_state => ReasonReact.Update({params: params}))
      },
    render: self => {
      let onSubmit = params => {
        dispatch(Redux.Paginate(1));
        dispatch(Redux.SaveSearch(params));
        self.send(SetParams(params));
      };
      <div>
        <SearchFormRe inputs={state.savedSearch} onSubmit />
        {
          let offset = (state.pageNumber - 1) * Thumbnails.pageSize;
          let queryParams = makeSearchParams(self.state.params);
          let query =
            SearchAssets.make(
              ~params=queryParams,
              ~pageSize=Thumbnails.pageSize,
              ~offset,
              (),
            );
          <SearchAssetsQuery variables=query##variables>
            ...{
                 ({result}) =>
                   switch (result) {
                   | Loading => <div> {ReasonReact.string("Loading")} </div>
                   | Error(error) =>
                     Js.log(error);
                     <div> {ReasonReact.string(error##message)} </div>;
                   | Data(response) =>
                     <Thumbnails.Component
                       state
                       dispatch
                       search=response##search
                     />
                   }
               }
          </SearchAssetsQuery>;
        }
      </div>;
    },
  };
};

/*
 * Order matters: keep this module definition below SearchRe, otherwise there
 * will be a peculiar compiler error about SearchRe.state escaping its scope.
 */
module SearchProvider = {
  let lens = Reductive.Lens.make((state: Redux.appState) => state);
  let make = Reductive.Provider.createMake(Redux.store, lens);
};

module Component = {
  let component = ReasonReact.statelessComponent("SearchComp");
  let make = _children => {
    ...component,
    render: _self => <div> <SearchProvider component=SearchRe.make /> </div>,
  };
};