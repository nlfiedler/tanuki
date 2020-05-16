//
// Copyright (c) 2020 Nathan Fiedler
//

[@bs.new] external makeFormData: unit => Fetch.formData = "FormData";

[@bs.send]
external appendFileData: (Fetch.formData, string, FileReader.File.t) => unit =
  "append";

let clickFileInput = [%raw
  {|
    function () {
      document.getElementById('magic-file-input').click();
    }
  |}
];

let sendOneFile = file => {
  let formData = makeFormData();
  appendFileData(formData, "asset", file);

  Js.Promise.(
    Fetch.fetchWithInit(
      "/import",
      Fetch.RequestInit.make(
        ~method_=Post,
        ~body=Fetch.BodyInit.makeWithFormData(formData),
        (),
      ),
    )
    |> then_(Fetch.Response.json)
  );
};

module Component = {
  type state = {
    pendingFiles: list(FileReader.File.t),
    nextFile: option(FileReader.File.t),
  };
  type action =
    | SetPending(list(FileReader.File.t))
    | StartUpload;
  [@react.component]
  let make = () => {
    let (state, dispatch) =
      React.useReducer(
        (state, action) =>
          switch (action) {
          | SetPending(files) => {...state, pendingFiles: files}
          | StartUpload =>
            switch (state.pendingFiles) {
            | [] => {...state, nextFile: None}
            | [hd, ...rest] => {pendingFiles: rest, nextFile: Some(hd)}
            }
          },
        {pendingFiles: [], nextFile: None},
      );
    let uploadButtonDisabled =
      List.length(state.pendingFiles) == 0
      || Belt.Option.isSome(state.nextFile);
    let dropzoneDisabled = Belt.Option.isSome(state.nextFile);
    let uploadButtonValue =
      Belt.Option.isSome(state.nextFile) ? "Uploading..." : "Upload";
    Belt.Option.forEach(state.nextFile, file =>
      sendOneFile(file)
      |> Js.Promise.then_(_ => dispatch(StartUpload) |> Js.Promise.resolve)
      |> ignore
    );
    <ReactDropzone
      multiple=true
      disabled=dropzoneDisabled
      onDrop={(acceptedFiles, _) =>
        dispatch(SetPending(Array.to_list(acceptedFiles)))
      }>
      {({getInputProps, getRootProps}) => {
         let inputProps = getInputProps();
         let rootProps = getRootProps();
         <div className="container">
           <div className="notification has-text-centered">
             {ReasonReact.string(
                "Choose files to upload and click the Upload button below.",
              )}
           </div>
           <div
             style={ReactDOMRe.Style.make(~marginBottom="1.5em", ())}
             className="dropzone"
             onBlur={rootProps.onBlur}
             onDragEnter={rootProps.onDragEnter}
             onDragLeave={rootProps.onDragLeave}
             onDragOver={rootProps.onDragOver}
             onDragStart={rootProps.onDragStart}
             onDrop={rootProps.onDrop}
             onFocus={rootProps.onFocus}
             onKeyDown={rootProps.onKeyDown}
             ref={ReactDOMRe.Ref.callbackDomRef(rootProps.ref)}
             tabIndex={rootProps.tabIndex}>
             <div> {React.string("You can drag and drop files here")} </div>
             <button type_="button" onClick=clickFileInput>
               {React.string("Open File Dialog")}
             </button>
             <input
               id="magic-file-input"
               autoComplete={inputProps.autoComplete}
               multiple={inputProps.multiple}
               onChange={inputProps.onChange}
               onClick={inputProps.onClick}
               ref={ReactDOMRe.Ref.callbackDomRef(inputProps.ref)}
               style={inputProps.style}
               tabIndex={inputProps.tabIndex}
               type_={inputProps.type_}
             />
           </div>
           <div className="columns">
             <div className="column is-two-thirds">
               <h5 className="subtitle">
                 {React.string("Files to be uploaded:")}
               </h5>
               <ul>
                 {ReasonReact.array(
                    Array.map(
                      (entry: FileReader.File.t) => {
                        let name = FileReader.File.name(entry);
                        <li key=name> {ReasonReact.string(name)} </li>;
                      },
                      Array.of_list(state.pendingFiles),
                    ),
                  )}
               </ul>
             </div>
             <div className="column is-one-third">
               <div className="field is-grouped is-grouped-right">
                 <div className="control">
                   <input
                     className="button is-primary"
                     type_="submit"
                     value=uploadButtonValue
                     onClick={_ => dispatch(StartUpload)}
                     disabled=uploadButtonDisabled
                   />
                 </div>
               </div>
             </div>
           </div>
         </div>;
       }}
    </ReactDropzone>;
  };
};