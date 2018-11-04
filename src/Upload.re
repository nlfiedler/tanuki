module UploadAsset = [%graphql
  {|
    mutation Upload($file: Upload!) {
      upload(file: $file)
    }
  |}
];

module UploadAssetMutation = ReasonApollo.CreateMutation(UploadAsset);

type validityState = {. "valid": bool};

/* Web File API object; different browsers have different properties. */
type fileType = {
  .
  "lastModified": int,
  "lastModifiedDate": string,
  "name": string,
  "size": int,
  "type_": string,
  "webkitRelativePath": string,
};

/* let fileToJson = (file: fileType): Js.Json.t =>
   Js.Json.(
     object_(
       Js.Dict.fromList([
         ("lastModified", file##lastModified |> float_of_int |> number),
         ("lastModifiedDate", file##lastModifiedDate |> string),
         ("name", file##name |> string),
         ("size", file##size |> float_of_int |> number),
         ("type", file##type_ |> string),
         ("webkitRelativePath", file##webkitRelativePath |> string),
       ]),
     )
   ); */

let uploadSelection =
    (event: ReactEvent.Form.t): (option(string), option(fileType)) => {
  let validity: validityState = ReactEvent.Form.target(event)##validity;
  if (!validity##valid) {
    (None, None);
  } else {
    let files: list(fileType) = ReactEvent.Form.target(event)##files;
    let firstFile = List.length(files) > 0 ? Some(List.hd(files)) : None;
    let filename: string = ReactEvent.Form.target(event)##value;
    /*
     * The browser adds a fake path onto the file name; this is only for viewing
     * purposes, the upload form will handle reading and uploading the file.
     */
    if (Js.String.startsWith("C:\\fakepath\\", filename)) {
      (Some(Js.String.sliceToEnd(~from=12, filename)), firstFile);
    } else {
      (Some(filename), firstFile);
    };
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

/* HACK: run the GraphQL mutation using JavaScript. */
let runMutate: (UploadAssetMutation.apolloMutation, fileType) => unit = [%raw
  (mutate, file) => "{ mutate({ file }) }"
];

/*
 * After the upload to /import, the backend redirects the browser to the asset
 * edit page, and that is handled in the main module.
 */
module Component = {
  type state = {
    uploadFilename: option(string),
    uploadFile: option(fileType),
  };
  type action =
    | UploadSelection((option(string), option(fileType)));
  let component = ReasonReact.reducerComponent("Upload");
  let make = _children => {
    ...component,
    initialState: () => {uploadFilename: None, uploadFile: None},
    reducer: action =>
      switch (action) {
      | UploadSelection((filename, file)) => (
          _state =>
            ReasonReact.Update({uploadFilename: filename, uploadFile: file})
        )
      },
    render: self => {
      let filename =
        Belt.Option.getWithDefault(self.state.uploadFilename, "");
      let helpDisplay = hasDraggable ? "block" : "none";
      let uploadDisabled = Belt.Option.isNone(self.state.uploadFilename);
      <UploadAssetMutation>
        ...{
             (mutate, {result}) =>
               switch (result) {
               | Loading => <p> {ReasonReact.string("Loading...")} </p>
               | Error(error) =>
                 Js.log(error);
                 <div> {ReasonReact.string(error##message)} </div>;
               | Data(result) =>
                 <EditAsset.Component assetId=result##upload />
               | NotCalled =>
                 <div>
                   <form action="#" method="post">
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
                             onChange=(
                               evt =>
                                 self.send(
                                   UploadSelection(uploadSelection(evt)),
                                 )
                             )
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
                         style={
                           ReactDOMRe.Style.make(~display=helpDisplay, ())
                         }>
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
                           onClick=(
                             _ => {
                               let file =
                                 Belt.Option.getExn(self.state.uploadFile);
                               runMutate(mutate, file);
                               /* let upload =
                                    UploadAsset.make(
                                      ~file=
                                        fileToJson(
                                          Belt.Option.getExn(
                                            self.state.uploadFile,
                                          ),
                                        ),
                                      (),
                                    );
                                  mutate(~variables=upload##variables, ())
                                  |> ignore; */
                             }
                           )
                         />
                       </div>
                     </div>
                   </form>
                 </div>
               }
           }
      </UploadAssetMutation>;
    },
  };
};