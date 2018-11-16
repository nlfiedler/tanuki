/* The expected shape of the thumbnail data from GraphQL. */
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
      "thumbnailUrl": string,
    }),
};

let pageSize = 18;
let rowWidth = 3;

let formatDate = (datetime: Js.Json.t) =>
  switch (Js.Json.decodeNumber(datetime)) {
  | None => "INVALID DATE"
  | Some(num) =>
    let d = Js.Date.fromFloat(num);
    Js.Date.toLocaleString(d);
  };

let namedPaginationLink = (label, page, pager) =>
  Some(
    <li key={string_of_int(page)}>
      <a onClick={_ => pager(page)}> {ReasonReact.string(label)} </a>
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
    currentPage - 10 <= 1 ?
      None : namedPaginationLink("«", currentPage - 10, pager);
  let preDots =
    lower > 2 ?
      Some(
        <li key="prev" className="pagination-ellipsis">
          <span> {ReasonReact.string("…")} </span>
        </li>,
      ) :
      None;
  let postDots =
    upper < totalPages - 1 ?
      Some(
        <li key="next" className="pagination-ellipsis">
          <span> {ReasonReact.string("…")} </span>
        </li>,
      ) :
      None;
  let nextLink =
    currentPage + 10 >= totalPages ?
      None : namedPaginationLink("»", currentPage + 10, pager);
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
  let component = ReasonReact.statelessComponent("Paging");
  let make = (~current: int, ~total: int, ~dispatch, _children) => {
    ...component,
    render: _self => {
      let setPage = (page: int) => dispatch(Redux.Paginate(page));
      <nav className="pagination is-centered" role="navigation">
        {makeLinks(current, total, setPage)}
      </nav>;
    },
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

module ThumbCard = {
  type state = {thumbless: bool};
  type action =
    | MarkThumbless;
  let component = ReasonReact.reducerComponent("Thumbnail");
  let make = (~entry, _children) => {
    ...component,
    initialState: () => {thumbless: false},
    reducer: action =>
      switch (action) {
      | MarkThumbless => (_state => ReasonReact.Update({thumbless: true}))
      },
    render: self =>
      <div key=entry##id className="column is-one-third">
        <div className="card">
          <div
            className="card-content"
            onClick={_ => ReasonReact.Router.push("/assets/" ++ entry##id)}>
            <figure
              className="image"
              style={
                /*
                 * overflow only works on block elements, so apply it here;
                 * long file names with "break" characters (e.g. '-') will
                 * wrap automatically anyway
                 */
                ReactDOMRe.Style.make(
                  ~overflow="hidden",
                  (),
                )
              }>
              {
                self.state.thumbless ?
                  <img
                    src={
                      "/images/"
                      ++ brokenThumbnailPlaceholder(entry##filename)
                    }
                    alt=entry##filename
                    style={ReactDOMRe.Style.make(~width="auto", ())}
                  /> :
                  <img
                    src=entry##thumbnailUrl
                    alt=entry##filename
                    onError={_ => self.send(MarkThumbless)}
                    style={ReactDOMRe.Style.make(~width="auto", ())}
                  />
              }
              <small>
                {ReasonReact.string(formatDate(entry##datetime))}
              </small>
              <br />
              <small> {ReasonReact.string(entry##filename)} </small>
            </figure>
          </div>
        </div>
      </div>,
  };
};

let makeCards = results =>
  Array.mapi(
    (i, a) => <ThumbCard key={string_of_int(i)} entry=a />,
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
  let component = ReasonReact.statelessComponent("Thumbnails");
  let make = (~state: Redux.appState, ~dispatch, ~search: t, _children) => {
    ...component,
    render: _self => {
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
        {
          if (search##count > pageSize && Array.length(search##results) > 0) {
            <Paging current={state.pageNumber} total=search##count dispatch />;
          } else {
            <span />;
          }
        }
      </div>;
    },
  };
};