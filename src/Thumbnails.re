//
// Copyright (c) 2020 Nathan Fiedler
//
type t = {
  .
  "count": int,
  "results":
    Js.Array.t({
      .
      "datetime": Js.Json.t,
      "filename": string,
      "id": string,
      "location": option(string),
      "mimetype": string,
    }),
};

let pageSize = 18;
let rowWidth = 3;

let formatDate = (datetime: Js.Json.t) =>
  switch (Js.Json.decodeString(datetime)) {
  | None => "INVALID STRING"
  | Some(dateStr) =>
    let d = Js.Date.fromString(dateStr);
    Js.Date.toLocaleString(d);
  };

let namedPaginationLink = (label, page, pager) =>
  Some(
    <li key={string_of_int(page)}>
      <a onClick={_ => pager(page)} className="pagination-link">
        {ReasonReact.string(label)}
      </a>
    </li>,
  );

let paginationLink = (currentPage: int, page: int, pager) => {
  let pageLabel = string_of_int(page);
  let className =
    "pagination-link" ++ (currentPage == page ? " is-current" : "");
  Some(
    <li key=pageLabel>
      <a onClick={_ => pager(page)} className>
        {ReasonReact.string(pageLabel)}
      </a>
    </li>,
  );
};

/* Convert the optional list elements into a list of React elements. */
let justLinksExtractor = maybeLinks => {
  let somes = List.filter(e => Js.Option.isSome(e), maybeLinks);
  let values = List.map(e => Js.Option.getWithDefault(<span />, e), somes);
  let valueToElem =
      (value: ReasonReact.reactElement): ReasonReact.reactElement => value;
  List.map(e => valueToElem(e), values);
};

let makeLinks = (currentPage: int, totalCount: int, pager) => {
  let totalPages =
    int_of_float(ceil(float_of_int(totalCount) /. float_of_int(pageSize)));
  let desiredLower = currentPage - 5;
  let desiredUpper = currentPage + 4;
  let (lower, upper) =
    if (desiredLower <= 1) {
      (2, min(desiredUpper + abs(desiredLower), totalPages - 1));
    } else if (desiredUpper >= totalPages) {
      (max(desiredLower - (desiredUpper - totalPages), 2), totalPages - 1);
    } else {
      (desiredLower, desiredUpper);
    };
  let numberedLinks = ref([]);
  for (pageNum in lower to upper) {
    let link = paginationLink(currentPage, pageNum, pager);
    numberedLinks := [link, ...numberedLinks^];
  };
  let firstLink = paginationLink(currentPage, 1, pager);
  let prevLink =
    currentPage - 10 <= 1
      ? None : namedPaginationLink("<<", currentPage - 10, pager);
  let preDots =
    lower > 2
      ? Some(
          <li key="prev" className="pagination-ellipsis">
            <span> {ReasonReact.string("...")} </span>
          </li>,
        )
      : None;
  let postDots =
    upper < totalPages - 1
      ? Some(
          <li key="next" className="pagination-ellipsis">
            <span> {ReasonReact.string("...")} </span>
          </li>,
        )
      : None;
  let nextLink =
    currentPage + 10 >= totalPages
      ? None : namedPaginationLink(">>", currentPage + 10, pager);
  let lastLink = paginationLink(currentPage, totalPages, pager);
  let maybeLinks =
    [firstLink, prevLink, preDots]
    @ List.rev(numberedLinks^)
    @ [postDots, nextLink, lastLink];
  <ul className="pagination-list">
    {ReasonReact.array(Array.of_list(justLinksExtractor(maybeLinks)))}
  </ul>;
};

module Paging = {
  [@react.component]
  let make = (~current: int, ~total: int, ~dispatch) => {
    let setPage = (page: int) => dispatch(Redux.Paginate(page));
    <nav
      className="pagination is-centered"
      role="navigation"
      style={ReactDOMRe.Style.make(~marginBottom="3em", ())}>
      {makeLinks(current, total, setPage)}
    </nav>;
  };
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
    <div className="column is-one-third">
      <figure
        className="image"
        onClick={_ => ReasonReact.Router.push("/assets/" ++ entry##id)}
        /*
         * overflow only works on block elements, so apply it here;
         * long file names with "break" characters (e.g. '-') will
         * wrap automatically anyway
         */
        style={ReactDOMRe.Style.make(
          ~cursor="pointer",
          ~overflow="hidden",
          (),
        )}>
        {state.thumbless
           ? <img
               src={"/images/" ++ brokenThumbnailPlaceholder(entry##filename)}
               alt=entry##filename
               style={ReactDOMRe.Style.make(~width="auto", ())}
             />
           : (
             if (Js.String.startsWith("video/", entry##mimetype)) {
               <video width="300" controls=true preload="auto">
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
                 src={"/api/thumbnail/300/300/" ++ entry##id}
                 alt=entry##filename
                 onError={_ => dispatch(MarkThumbless)}
                 style={ReactDOMRe.Style.make(~width="auto", ())}
               />;
             }
           )}
        <small> {ReasonReact.string(formatDate(entry##datetime))} </small>
        <br />
        <small> {ReasonReact.string(entry##filename)} </small>
      </figure>
    </div>;
  };
};

let makeCards = results =>
  Array.map(
    entry =>
      <ThumbCard
        key={
          entry##id;
        }
        entry
      />,
    results,
  );

let makeRows = cards => {
  let idx = ref(0);
  let rows = ref([]);
  while (idx^ < Js.Array.length(cards)) {
    let nextIdx = idx^ + rowWidth;
    let row = Js.Array.slice(~start=idx^, ~end_=nextIdx, cards);
    rows := [row, ...rows^];
    idx := nextIdx;
  };
  /* would like to have a List.rev_mapi()... */
  List.mapi(
    (ii, row) =>
      <div key={string_of_int(ii)} className="columns">
        {ReasonReact.array(row)}
      </div>,
    List.rev(rows^),
  );
};

module Component = {
  [@react.component]
  let make = (~state: Redux.appState, ~dispatch, ~search: t) => {
    let cards = makeCards(search##results);
    let rows = makeRows(cards);
    /*
     * Use the awesome, flexible bulma columns and then force them
     * to be thirds, so we avoid the images shrinking needlessly.
     * Then wrap the individual column elements in a "columns",
     * one for each row, and that is collected in a container.
     * Basically a primitive table.
     */
    <div className="container">
      {ReasonReact.array(Array.of_list(rows))}
      {if (search##count > pageSize && Array.length(search##results) > 0) {
         <Paging current={state.pageNumber} total=search##count dispatch />;
       } else {
         <span />;
       }}
    </div>;
  };
};