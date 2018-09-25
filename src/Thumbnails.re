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

module ThumbCard = {
  /* TODO: become a reducer component, listen for onError, switch to placeholder thumbnail */
  /*
   -- any asset that fails to produce a thumbnail will get a placeholder
   imgSrc =
       if entry.thumbless then
           "/images/" ++ brokenThumbnailPlaceholder entry.file_name
       else
           "/thumbnail/" ++ entry.id
   imgAttrs =
       if entry.thumbless then
           baseImgAttrs
       else
           baseImgAttrs ++ [ on "error" (Json.Decode.succeed (ThumblessAsset entry.id)) ]
       ] */
  let component = ReasonReact.statelessComponent("Thumbnail");
  let make = (~entry, _children) => {
    ...component,
    render: _self =>
      <div className="column is-one-third">
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
              <img
                src={"/thumbnail/" ++ entry##id}
                alt=entry##filename
                style={ReactDOMRe.Style.make(~width="auto", ())}
              />
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
  List.map(
    row => <div className="columns"> {ReasonReact.array(row)} </div>,
    List.rev(rows^),
  );
};

module Component = {
  let component = ReasonReact.statelessComponent("Thumbnails");
  let make = (~search: t, _children) => {
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
      <div className="container"> ...{Array.of_list(rows)} </div>;
    },
  };
};