let uploadSelection = (event: ReactEvent.Form.t): string => {
  let filename: string = ReactEvent.Form.target(event)##value;
  /*
   * The browser adds a fake path onto the file name; this is only for viewing
   * purposes, the upload form will handle reading and uploading the file.
   */
  if (Js.String.startsWith("C:\\fakepath\\", filename)) {
    Js.String.sliceToEnd(~from=12, filename);
  } else {
    filename;
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
 * After the upload to /import, the backend redirects the browser to the asset
 * edit page, and that is handled in the main module.
 */
module Component = {
  type state = {uploadFilename: option(string)};
  type action =
    | UploadSelection(string);
  let component = ReasonReact.reducerComponent("Upload");
  let make = _children => {
    ...component,
    initialState: () => {uploadFilename: None},
    reducer: action =>
      switch (action) {
      | UploadSelection(filename) => (
          _state => ReasonReact.Update({uploadFilename: Some(filename)})
        )
      },
    render: self => {
      let filename =
        Belt.Option.getWithDefault(self.state.uploadFilename, "");
      let helpDisplay = hasDraggable ? "block" : "none";
      let uploadDisabled = Belt.Option.isNone(self.state.uploadFilename);
      <div>
        <form action="/import" method="post" encType="multipart/form-data">
          <div className="control">
            <div className="file has-name is-boxed">
              <label className="file-label">
                <input
                  className="file-input"
                  id="fileInput"
                  type_="file"
                  multiple=false
                  name="asset"
                  required=true
                  onChange={
                    evt => self.send(UploadSelection(uploadSelection(evt)))
                  }
                />
                <span className="file-cta">
                  <span className="file-icon">
                    <i className="fas fa-upload" />
                  </span>
                  <span className="file-label">
                    {ReasonReact.string("Choose a file...")}
                  </span>
                </span>
                <span className="file-name">
                  {ReasonReact.string(filename)}
                </span>
              </label>
            </div>
            <p
              className="help"
              style={ReactDOMRe.Style.make(~display=helpDisplay, ())}>
              {
                ReasonReact.string(
                  "You can drag and drop a file on the above control",
                )
              }
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
      </div>;
    },
  };
};