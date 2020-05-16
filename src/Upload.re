//
// Copyright (c) 2020 Nathan Fiedler
//
type validityState = {. "valid": bool};

let uploadSelection = (event: ReactEvent.Form.t): list(string) => {
  let validity: validityState = ReactEvent.Form.target(event)##validity;
  if (!validity##valid) {
    [];
  } else {
    // The list of files is a special object of type FileList with a
    // `length` property and an `item(N)` method.
    // c.f. https://developer.mozilla.org/en-US/docs/Web/API/FileList
    let files = ReactEvent.Form.target(event)##files;
    // Each File entry has name, size, type (mime type), lastModified.
    // c.f. https://developer.mozilla.org/en-US/docs/Web/API/File
    let getName = (file: Js.Json.t): option(Js.Json.t) => {
      switch (Js.Json.decodeObject(file)) {
      | Some(obj) => Js.Dict.get(obj, "name")
      | None => None
      };
    };
    // extract just the file names from the file list
    let filenames = ref([]);
    for (idx in 0 to files##length) {
      let row = getName(files##item(idx));
      filenames := [row, ...filenames^];
    };
    // discard anything that wasn't a valid json file entry
    List.fold_right(
      (a: option(Js.Json.t), b: list(string)): list(string) =>
        switch (a) {
        | Some(name) =>
          switch (Js.Json.decodeString(name)) {
          | Some(str) => [str, ...b]
          | None => b
          }
        | None => b
        },
      filenames^,
      [],
    );
  };
};

/*
 * Hack to detect if the browser supports drag and drop. This is not perfect,
 * but it is better than nothing.
 */
let hasDraggable = [%raw
  {|
    function () {
      var div = document.createElement('div');
      return ('draggable' in div) || ('ondragstart' in div && 'ondrop' in div);
    }
  |}
];

/*
 * After the upload to /import, the backend redirects the browser to the recent
 * imports page, and that is handled in the main module.
 */
module Component = {
  type state = {uploadFilenames: list(string)};
  type action =
    | UploadSelection(list(string));
  [@react.component]
  let make = () => {
    let (state, dispatch) =
      React.useReducer(
        (_state, action) =>
          switch (action) {
          | UploadSelection(filenames) => {uploadFilenames: filenames}
          },
        {uploadFilenames: []},
      );
    let helpDisplay = hasDraggable ? "block" : "none";
    let uploadDisabled = List.length(state.uploadFilenames) == 0;
    <div>
      <div className="columns">
        <div className="column is-one-third">
          <form action="/import" method="post" encType="multipart/form-data">
            <div className="control">
              <div className="file has-name is-boxed">
                <label className="file-label">
                  <input
                    className="file-input"
                    id="fileInput"
                    type_="file"
                    multiple=true
                    name="asset"
                    required=true
                    onChange={evt =>
                      dispatch(UploadSelection(uploadSelection(evt)))
                    }
                  />
                  <span className="file-cta">
                    <span className="file-icon">
                      <i className="fas fa-upload" />
                    </span>
                    <span className="file-label">
                      {ReasonReact.string("Choose files...")}
                    </span>
                  </span>
                </label>
              </div>
              <p
                className="help"
                style={ReactDOMRe.Style.make(~display=helpDisplay, ())}>
                {ReasonReact.string(
                   "You can drag and drop files on the above control",
                 )}
              </p>
              <div className="control">
                <input
                  className="button is-primary"
                  type_="submit"
                  value="Upload"
                  disabled=uploadDisabled
                />
              </div>
            </div>
          </form>
        </div>
        <div className="column is-two-thirds">
          {List.length(state.uploadFilenames) == 0
             ? React.null
             : <h1 className="title">
                 {ReasonReact.string("Files to be uploaded:")}
               </h1>}
          {ReasonReact.array(
             {Array.map(
                (entry: string) =>
                  <p key=entry> {ReasonReact.string(entry)} </p>,
                Array.of_list(state.uploadFilenames),
              )},
           )}
        </div>
      </div>
    </div>;
  };
};