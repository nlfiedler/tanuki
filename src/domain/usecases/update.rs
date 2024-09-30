//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Asset, AssetInput, Location};
use crate::domain::repositories::RecordRepository;
use anyhow::Error;
use std::cmp;
use std::fmt;

///
/// Update an existing asset with new values, merging with the current record,
/// storing in the data repository, and returning the result.
///
pub struct UpdateAsset {
    records: Box<dyn RecordRepository>,
}

impl UpdateAsset {
    pub fn new(records: Box<dyn RecordRepository>) -> Self {
        Self { records }
    }
}

impl super::UseCase<Asset, Params> for UpdateAsset {
    fn call(&self, params: Params) -> Result<Asset, Error> {
        // fetch existing record to merge with incoming values
        let mut asset = self.records.get_asset(&params.asset.key)?;
        // merge the incoming values with the existing record
        merge_asset_input(&mut asset, params.asset);
        // store the updated record in the repository
        self.records.put_asset(&asset)?;
        Ok(asset)
    }
}

#[derive(Clone)]
pub struct Params {
    asset: AssetInput,
}

impl Params {
    pub fn new(asset: AssetInput) -> Self {
        Self { asset }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({:?})", self.asset.key)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.asset.key == other.asset.key
    }
}

impl cmp::Eq for Params {}

fn merge_asset_input(asset: &mut Asset, input: AssetInput) {
    if let Some(tags) = input.tags {
        // incoming tags replace existing tags
        let mut groomed = tags.clone();
        // Filter out empty tags, as the front-end may send those because it is
        // too lazy to filter them itself.
        groomed.retain(|t| !t.is_empty());
        groomed.sort();
        groomed.dedup();
        asset.tags = groomed;
    }
    if let Some(filename) = input.filename {
        if !filename.is_empty() {
            asset.filename = filename;
        }
    }
    // merge the existing and new location, if any, and save if changed
    let location = super::merge_locations(asset.location.clone(), input.location.clone());
    if location.is_some() {
        if let Some(ref loc) = location {
            // largely a symbolic gesture as the serializer will turn the empty
            // location structure into a single null anyway, but for clarity we
            // wipe out the location if the input is completey blank
            if loc.label.is_none() && loc.city.is_none() && loc.region.is_none() {
                asset.location = None
            } else {
                asset.location = location;
            }
        }
    }
    // parse the caption to glean location and additional tags
    if let Some(caption) = input.caption {
        asset.caption = Some(caption.clone());
        let result = caption::lex(&caption);
        // tags in the caption are merged with the asset/input tags
        asset.tags.extend_from_slice(&result.tags[..]);
        asset.tags.sort();
        asset.tags.dedup();
        if asset.location.is_none() {
            // do not overwrite current location if it is already set
            asset.location = result.location.map(|v| Location::new(&v));
        }
    }
    // permit user to update/remove the custom date/time
    asset.user_date = input.datetime;
    // do not overwrite media_type with null/blank values
    if let Some(mt) = input.media_type {
        if !mt.is_empty() {
            asset.media_type = mt.to_lowercase();
        }
    }
}

mod caption {
    use std::str::CharIndices;

    /// The `Lexer` struct holds the state of the lexical analyzer.
    pub struct Lexer<'a> {
        // iterator of the characters in the string
        iter: CharIndices<'a>,
        // the next character to return, if peek() has been called
        peeked: Option<(usize, char)>,
        // position of next character to read (in bytes)
        pos: usize,
        // width of last character read from input (in bytes)
        width: usize,
        // collects the results of the lexical analysis
        results: Result,
    }

    #[derive(Default)]
    pub struct Result {
        pub tags: Vec<String>,
        pub location: Option<String>,
    }

    impl<'a> Lexer<'a> {
        /// `new` constructs an instance of `Lexer` for the named input.
        fn new(input: &'a str) -> Lexer<'a> {
            Lexer {
                iter: input.char_indices(),
                peeked: None,
                pos: 0,
                width: 0,
                results: Default::default(),
            }
        }

        /// `next` returns the next rune in the input, or `None` if at the end.
        fn next(&mut self) -> Option<char> {
            let next = if self.peeked.is_some() {
                self.peeked.take()
            } else {
                self.iter.next()
            };
            match next {
                Some((pos, ch)) => {
                    self.width = ch.len_utf8();
                    self.pos = pos + self.width;
                    Some(ch)
                }
                None => None,
            }
        }

        /// `peek` returns but does not consume the next rune in the input.
        fn peek(&mut self) -> Option<char> {
            if self.peeked.is_none() {
                self.peeked = self.iter.next();
            }
            self.peeked.map(|(_, ch)| ch)
        }
    }

    struct StateFn(fn(&mut Lexer) -> Option<StateFn>);

    /// Runs the lexical analysis on the text and returns the results.
    pub fn lex(input: &str) -> Result {
        let mut lexer = Lexer::new(input);
        // inform the compiler what the type of state _really_ is
        let mut state: fn(&mut Lexer) -> Option<StateFn> = lex_start;
        while let Some(next) = state(&mut lexer) {
            let StateFn(state_fn) = next;
            state = state_fn;
        }
        lexer.results
    }

    fn lex_start(l: &mut Lexer) -> Option<StateFn> {
        while let Some(ch) = l.next() {
            if ch == '#' {
                return Some(StateFn(lex_tag));
            } else if ch == '@' {
                return Some(StateFn(lex_location));
            }
        }
        None
    }

    fn lex_tag(l: &mut Lexer) -> Option<StateFn> {
        let tag = lex_identifier(l);
        l.results.tags.push(tag);
        Some(StateFn(lex_start))
    }

    fn lex_location(l: &mut Lexer) -> Option<StateFn> {
        if let Some(ch) = l.peek() {
            if ch == '"' {
                // ignore the opening quote
                l.next();
                // scan until the next quote is found
                let mut ident = String::new();
                while let Some(ch) = l.peek() {
                    if ch == '"' {
                        break;
                    } else {
                        ident.push(ch);
                        l.next();
                    }
                }
                l.results.location = Some(ident);
            } else {
                let location = lex_identifier(l);
                l.results.location = Some(location);
            }
        }
        Some(StateFn(lex_start))
    }

    /// `lex_identifier` processes the text as a tag or location.
    fn lex_identifier(l: &mut Lexer) -> String {
        let mut ident = String::new();
        while let Some(ch) = l.peek() {
            if is_delimiter(ch) {
                break;
            } else {
                ident.push(ch);
                l.next();
            }
        }
        ident
    }

    /// `is_delimiter` returns true if `ch` is a delimiter character.
    fn is_delimiter(ch: char) -> bool {
        matches!(ch, ' ' | '.' | ',' | ';' | '(' | ')' | '"')
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_boring_caption() {
            let results = lex("this is a boring caption");
            assert_eq!(results.tags.len(), 0);
            assert!(results.location.is_none());
        }

        #[test]
        fn test_basic_caption() {
            let results = lex("#cat and #dog @hawaii");
            assert_eq!(results.tags.len(), 2);
            assert!(results.tags.iter().any(|l| l == "cat"));
            assert!(results.tags.iter().any(|l| l == "dog"));
            assert_eq!(results.location.unwrap(), "hawaii");

            let results = lex("#cat, #dog, #mouse");
            assert_eq!(results.tags.len(), 3);
            assert!(results.tags.iter().any(|l| l == "cat"));
            assert!(results.tags.iter().any(|l| l == "dog"));
            assert!(results.tags.iter().any(|l| l == "mouse"));
            assert!(results.location.is_none());
        }

        #[test]
        fn test_identifier_delimiters() {
            let results = lex("#cat. #dog, #bird #mouse; #house(#car)");
            assert_eq!(results.tags.len(), 6);
            assert!(results.tags.iter().any(|l| l == "cat"));
            assert!(results.tags.iter().any(|l| l == "dog"));
            assert!(results.tags.iter().any(|l| l == "bird"));
            assert!(results.tags.iter().any(|l| l == "mouse"));
            assert!(results.tags.iter().any(|l| l == "house"));
            assert!(results.tags.iter().any(|l| l == "car"));
            assert!(results.location.is_none());
        }

        #[test]
        fn test_quoted_location() {
            let results = lex("having #fun @\"the beach\"");
            assert_eq!(results.tags.len(), 1);
            assert!(results.tags[0] == "fun");
            assert_eq!(results.location.unwrap(), "the beach");

            // missing the closing quote
            let results = lex("having #fun @\"the beach");
            assert_eq!(results.tags.len(), 1);
            assert!(results.tags[0] == "fun");
            assert_eq!(results.location.unwrap(), "the beach");
        }

        #[test]
        fn test_parenthesis_combo() {
            // case where "(#nathan, #oma, #opa)" with the old JavaScript code
            // would yield ["oma", "opa)"]
            let results = lex("(#nathan, #oma, #opa)");
            assert_eq!(results.tags.len(), 3);
            assert!(results.tags.iter().any(|l| l == "nathan"));
            assert!(results.tags.iter().any(|l| l == "oma"));
            assert!(results.tags.iter().any(|l| l == "opa"));
            assert!(results.location.is_none());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Location;
    use crate::domain::repositories::MockRecordRepository;
    use anyhow::anyhow;
    use chrono::prelude::*;
    use mockall::predicate::*;

    fn make_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
    }

    #[test]
    fn test_merge_asset_input_noop() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: None,
            location: None,
            datetime: None,
            media_type: None,
            filename: None,
        };
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 1);
        assert_eq!(asset.tags[0], "kittens");
        assert!(asset.caption.is_none());
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "image/jpeg");
    }

    #[test]
    fn test_merge_asset_input_media_type() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: None,
            location: None,
            datetime: None,
            media_type: Some("video/quicktime".to_owned()),
            filename: None,
        };
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 1);
        assert_eq!(asset.tags[0], "kittens");
        assert!(asset.caption.is_none());
        assert!(asset.location.is_none());
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "video/quicktime");
    }

    #[test]
    fn test_merge_asset_input_no_clobber() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cute".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: Some("#kittens and #puppies @paris".to_owned()),
            location: None,
            datetime: None,
            media_type: None,
            filename: None,
        };
        // location in caption should not clobber an existing location value
        //
        // tags in caption should merge with existing tags
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 3);
        assert!(asset.tags.iter().any(|l| l == "cute"));
        assert!(asset.tags.iter().any(|l| l == "kittens"));
        assert!(asset.tags.iter().any(|l| l == "puppies"));
        assert_eq!(asset.caption.unwrap(), "#kittens and #puppies @paris");
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "image/jpeg");
        assert_eq!(asset.filename, "fighting_kittens.jpg");
    }

    #[test]
    fn test_merge_asset_input_no_clobber_blank() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cute".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: Some("#kittens and #puppies @paris".to_owned()),
            location: None,
            datetime: None,
            media_type: Some("".to_owned()),
            filename: Some("".to_owned()),
        };
        // blank filename and media type should not overwrite record
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 3);
        assert!(asset.tags.iter().any(|l| l == "cute"));
        assert!(asset.tags.iter().any(|l| l == "kittens"));
        assert!(asset.tags.iter().any(|l| l == "puppies"));
        assert_eq!(asset.caption.unwrap(), "#kittens and #puppies @paris");
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "image/jpeg");
        assert_eq!(asset.filename, "fighting_kittens.jpg");
    }

    #[test]
    fn test_merge_asset_input_tags_replace() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned(), "puppies".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: Some(vec![
                "kittens".to_owned(),
                "kittens".to_owned(),
                "kittens".to_owned(),
            ]),
            caption: None,
            location: None,
            datetime: None,
            media_type: None,
            filename: None,
        };
        // new tags should replace existing tags
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 1);
        assert_eq!(asset.tags[0], "kittens");
        assert!(asset.caption.is_none());
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "image/jpeg");
    }

    #[test]
    fn test_merge_asset_input_tags_caption() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cute".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: Some(vec!["puppies".to_owned()]),
            caption: Some("#kittens fighting #kittens".to_owned()),
            location: None,
            datetime: None,
            media_type: None,
            filename: None,
        };
        // Tags in caption are merged with existing tags, but incoming tags
        // still replace existing tags.
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 2);
        assert!(asset.tags.iter().any(|l| l == "kittens"));
        assert!(asset.tags.iter().any(|l| l == "puppies"));
        assert_eq!(asset.caption.unwrap(), "#kittens fighting #kittens");
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "image/jpeg");
    }

    #[test]
    fn test_merge_asset_input_set_userdate() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let user_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: None,
            location: None,
            datetime: Some(user_date),
            media_type: None,
            filename: None,
        };
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 1);
        assert_eq!(asset.tags[0], "kittens");
        assert!(asset.caption.is_none());
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert_eq!(asset.user_date.unwrap(), user_date);
        assert_eq!(asset.media_type, "image/jpeg");
    }

    #[test]
    fn test_merge_asset_input_clear_userdate() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: None,
            location: None,
            datetime: None,
            media_type: None,
            filename: None,
        };
        merge_asset_input(&mut asset, input);
        assert_eq!(asset.tags.len(), 1);
        assert_eq!(asset.tags[0], "kittens");
        assert!(asset.caption.is_none());
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert!(asset.user_date.is_none());
        assert_eq!(asset.media_type, "image/jpeg");
    }

    #[test]
    fn test_merge_asset_input_clear_location() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: None,
            location: Some(Location::new("".into())),
            datetime: None,
            media_type: None,
            filename: None,
        };
        merge_asset_input(&mut asset, input);
        assert!(asset.location.is_none());
    }

    #[test]
    fn test_update_asset_ok() {
        // arrange
        let user_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cute".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: Some(user_date),
            original_date: None,
            dimensions: None,
        };
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: Some(vec!["puppies".to_owned()]),
            caption: Some("#kittens fighting #kittens".to_owned()),
            location: None,
            datetime: Some(user_date),
            media_type: None,
            filename: Some("kittens_fighting.jpg".to_owned()),
        };
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        records.expect_put_asset().returning(move |_| Ok(()));
        // act
        let usecase = UpdateAsset::new(Box::new(records));
        let params = Params::new(input);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.location.unwrap().label.unwrap(), "hawaii");
        assert_eq!(asset.filename, "kittens_fighting.jpg");
        assert_eq!(asset.tags.len(), 2);
        assert!(asset.tags.iter().any(|l| l == "kittens"));
        assert!(asset.tags.iter().any(|l| l == "puppies"));
        assert_eq!(asset.caption.unwrap(), "#kittens fighting #kittens");
        assert_eq!(asset.user_date.unwrap(), user_date);
        assert_eq!(asset.media_type, "image/jpeg");
    }

    #[test]
    fn test_update_asset_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let usecase = UpdateAsset::new(Box::new(mock));
        let input = AssetInput {
            key: "abc123".to_owned(),
            tags: None,
            caption: None,
            location: None,
            datetime: None,
            media_type: None,
            filename: None,
        };
        let params = Params::new(input);
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
