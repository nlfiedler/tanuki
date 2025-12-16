//
// Copyright (c) 2025 Nathan Fiedler
//
import MIMEType from 'whatwg-mimetype';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AsyncQueue } from 'tanuki/server/shared/collections/async-queue.ts';

/** Determines if an asset matches certain criteria. */
interface Constraint {
  /** For a given asset, return `true` if the asset matches. */
  matches(asset: Asset): boolean;
}

/** Matches if both sides also match. */
class AndConstraint {
  lhs: Constraint;
  rhs: Constraint;

  constructor(lhs: Constraint, rhs: Constraint) {
    this.lhs = lhs;
    this.rhs = rhs;
  }

  matches(asset: Asset): boolean {
    return this.lhs.matches(asset) && this.rhs.matches(asset);
  }
}

/** Matches if either side matches. */
class OrConstraint {
  lhs: Constraint;
  rhs: Constraint;

  constructor(lhs: Constraint, rhs: Constraint) {
    this.lhs = lhs;
    this.rhs = rhs;
  }

  matches(asset: Asset): boolean {
    return this.lhs.matches(asset) || this.rhs.matches(asset);
  }
}

/** Mathces only if right-hand-side predicate does not match. */
class NotConstraint {
  rhs: Constraint;

  constructor(rhs: Constraint) {
    this.rhs = rhs;
  }

  matches(asset: Asset): boolean {
    return !this.rhs.matches(asset);
  }
}

/** Matches the filename field of the asset. */
class FilenameConstraint {
  name: string;

  constructor(name: string) {
    this.name = name.toLowerCase();
  }

  matches(asset: Asset): boolean {
    return this.name == asset.filename.toLowerCase();
  }
}

// Extended Backus–Naur form of media type:
// mime-type = type "/" [tree "."] subtype ["+" suffix] *[";" parameter];

/** Matches the 'type' of the mediaType field of the asset. */
class TypeConstraint {
  type: string;

  constructor(type: string) {
    this.type = type.toLowerCase();
  }

  matches(asset: Asset): boolean {
    const mimeType = new MIMEType(asset.mediaType);
    return this.type == mimeType.type;
  }
}

/** Matches the 'subtype' of the mediaType field of the asset. */
class SubtypeConstraint {
  subtype: string;

  constructor(subtype: string) {
    this.subtype = subtype.toLowerCase();
  }

  matches(asset: Asset): boolean {
    const mimeType = new MIMEType(asset.mediaType);
    return this.subtype == mimeType.subtype;
  }
}

/** Matches any of the values in the tag field of the asset. */
class TagConstraint {
  tag: string;

  constructor(tag: string) {
    this.tag = tag.toLowerCase();
  }

  matches(asset: Asset): boolean {
    return asset.tags.some((e) => e.toLowerCase() == this.tag);
  }
}

enum LocationField {
  Any,
  Label,
  City,
  Region
}

function stringToLocationField(s: string): LocationField {
  if (s == 'label') {
    return LocationField.Label;
  } else if (s == 'city') {
    return LocationField.City;
  } else if (s == 'region') {
    return LocationField.Region;
  } else if (s == 'any') {
    return LocationField.Any;
  } else {
    throw new Error("field must be 'any', 'label', 'city', or 'region'");
  }
}

// struct LocationPredicate(LocationField, String);
class LocationConstraint {
  field: LocationField;
  value: string;

  constructor(field: LocationField, value: string) {
    this.field = field;
    this.value = value.toLowerCase();
  }

  matches(asset: Asset): boolean {
    if (this.value.length === 0) {
      // corresponding field(s) must not have a value
      switch (this.field) {
        case LocationField.Any: {
          return (
            asset.location?.label == null ||
            asset.location?.city == null ||
            asset.location?.region == null
          );
        }
        case LocationField.Label: {
          return asset.location?.label == null;
        }
        case LocationField.City: {
          return asset.location?.city == null;
        }
        case LocationField.Region: {
          return asset.location?.region == null;
        }
      }
    } else {
      // corresponding field(s) must have a matching value
      switch (this.field) {
        case LocationField.Any: {
          return asset.location?.partialMatch(this.value) ?? false;
        }
        case LocationField.Label: {
          return asset.location?.label?.toLowerCase() === this.value;
        }
        case LocationField.City: {
          return asset.location?.city?.toLowerCase() === this.value;
        }
        case LocationField.Region: {
          return asset.location?.region?.toLowerCase() === this.value;
        }
      }
    }
  }
}

/** Matches if the asset's best date is after the one given. */
class AfterConstraint {
  after: Date;

  constructor(after: Date) {
    this.after = after;
  }

  matches(asset: Asset): boolean {
    return asset.bestDate() >= this.after;
  }
}

/** Matches if the asset's best date is before the one given. */
class BeforeConstraint {
  before: Date;

  constructor(before: Date) {
    this.before = before;
  }

  matches(asset: Asset): boolean {
    return asset.bestDate() < this.before;
  }
}

/** An empty constraint that matches nothing. */
class EmptyConstraint {
  matches(asset: Asset): boolean {
    return false;
  }
}

/** Convert a keyword and its arguments into a constraint. */
function buildPredicate(atom: string[]): Constraint {
  const keyword = atom.shift() ?? 'undefined';
  if (keyword == 'after') {
    return new AfterConstraint(new Date(atom.shift()!));
  } else if (keyword == 'before') {
    return new BeforeConstraint(new Date(atom.shift()!));
  } else if (keyword == 'is') {
    return new TypeConstraint(atom.shift()!);
  } else if (keyword == 'format') {
    return new SubtypeConstraint(atom.shift()!);
  } else if (keyword == 'filename') {
    return new FilenameConstraint(atom.shift()!);
  } else if (keyword == 'loc') {
    if (atom.length == 1) {
      return new LocationConstraint(LocationField.Any, atom.shift()!);
    } else if (atom.length == 2) {
      const field = stringToLocationField(atom.shift()!);
      return new LocationConstraint(field, atom.shift()!);
    } else {
      throw new Error('loc: requires 1 or 2 arguments');
    }
  } else if (keyword == 'tag') {
    return new TagConstraint(atom.shift()!);
  } else {
    throw new Error(`unsupported predicate: ${keyword}`);
  }
}

/*
 * A simple parser for the query language, modeled after that of the
 * perkeep application (https://perkeep.org).
 */

/** Parse the given query and return a constraint for filtering assets. */
async function parse(query: string): Promise<Constraint> {
  const parser = new QueryParser(query);
  const result = await parser.parseExpression();
  await parser.drainLexer();
  return result;
}

class QueryParser {
  tokens: AsyncQueue<Token>;
  peeked: Token | null;

  constructor(query: string) {
    this.tokens = lex(query);
    this.peeked = null;
  }

  /**
   * Ensure all tokens are read from the queue such that the lexer can exit
   * properly and be garbage collected.
   */
  async drainLexer() {
    try {
      while (true) {
        const token = await this.tokens.dequeue();
        if (token.typ == TokenType.Eof) {
          break;
        }
      }
    } catch {
      // ignored, the queue closed
    }
  }

  async next(): Promise<Token | undefined> {
    if (this.peeked) {
      const p = this.peeked;
      this.peeked = null;
      return p;
    }
    return this.tokens.dequeue();
  }

  async peek(): Promise<Token | undefined> {
    if (this.peeked == null) {
      this.peeked = await this.tokens.dequeue();
    }
    return this.peeked;
  }

  /**
   * Parse the query from beginning to end, which includes expressions wrapped
   * in parentheses.
   */
  async parseExpression(): Promise<Constraint> {
    const p = await this.peek();
    if (p?.typ == TokenType.Eof) {
      return new EmptyConstraint();
    }
    let ret = await this.parseOperand();
    while (true) {
      const p = await this.peek();
      if (p?.typ == TokenType.And) {
        this.next();
      } else if (p?.typ == TokenType.Or) {
        this.next();
        return this.parseOrRhs(ret);
      } else if (p?.typ == TokenType.Close || p?.typ == TokenType.Eof) {
        break;
      }
      ret = await this.parseAndRhs(ret);
    }
    return ret;
  }

  /**
   * Process the next token as an operand, returning an error if it is
   * anything else.
   */
  async parseOperand(): Promise<Constraint> {
    const negated = await this.stripNot();
    let ret = new EmptyConstraint();
    const op = await this.peek();
    if (op?.typ == TokenType.Error) {
      throw new Error(op.val);
    } else if (op?.typ == TokenType.Eof) {
      throw new Error(`error: expected operand, got ${op.val}`);
    } else if (op?.typ == TokenType.Close) {
      throw new Error(`error: found ) without (, got ${op.val}`);
    } else if (
      op?.typ == TokenType.Predicate ||
      op?.typ == TokenType.Colon ||
      op?.typ == TokenType.Arg
    ) {
      ret = await this.parseAtom();
    } else if (op?.typ == TokenType.Open) {
      ret = await this.parseGroup();
    }
    if (negated) {
      ret = new NotConstraint(ret);
    }
    return ret;
  }

  /**
   * Processes consecuivee `not` operators, returning `true` if an odd number
   * and `false` otherwise.
   */
  async stripNot(): Promise<boolean> {
    let negated = false;
    while (true) {
      const p = await this.peek();
      if (p?.typ == TokenType.Not) {
        this.next();
        negated = !negated;
        continue;
      }
      break;
    }
    return negated;
  }

  /// Current token is expected to be a predicate, followed by a colon,
  /// and one or more arguments separated by colons. The predicate will be
  /// converted to one of the supported predicates.
  async parseAtom(): Promise<Constraint> {
    let i = await this.peek();
    const a: string[] = [];
    // confirm that the first token is a predicate, everything else is wrong
    if (i?.typ == TokenType.Predicate) {
      this.next();
      a.push(i.val);
    } else {
      throw new Error(`expected predicate, got ${i}`);
    }
    let arg_expected = false;
    while (true) {
      i = await this.peek();
      if (i?.typ == TokenType.Colon) {
        arg_expected = true;
        this.next();
        continue;
      } else if (i?.typ == TokenType.Arg) {
        arg_expected = false;
        i = await this.next();
        a.push(i!.val);
        continue;
      }
      if (arg_expected) {
        // inject an empty argument after the trailing colon
        a.push('');
      }
      break;
    }
    return buildPredicate(a);
  }

  /// Current token is expected to be an open paren.
  async parseGroup(): Promise<Constraint> {
    // confirm the next token is an open paren
    const i = await this.next();
    if (i?.typ == TokenType.Open) {
      const c = await this.parseExpression();
      const p = await this.peek();
      if (p?.typ == TokenType.Close) {
        this.next();
        return c;
      }
      throw new Error(`no matching ) at ${i}`);
    }
    throw new Error(`expected ( but got ${i}`);
  }

  /// Process the right side of the `or`, including chained `or` operators.
  async parseOrRhs(lhs: Constraint): Promise<Constraint> {
    let ret = lhs;
    while (true) {
      const rhs = await this.parseAnd();
      ret = new OrConstraint(ret, rhs);
      const p = await this.peek();
      if (p?.typ == TokenType.Or) {
        this.next();
      } else if (
        p?.typ == TokenType.And ||
        p?.typ == TokenType.Close ||
        p?.typ == TokenType.Eof
      ) {
        break;
      }
    }
    return ret;
  }

  /// Process the `and` and whatever comes after it.
  async parseAnd(): Promise<Constraint> {
    const ret = await this.parseOperand();
    const p = await this.peek();
    if (p?.typ == TokenType.And) {
      this.next();
    } else if (
      p?.typ == TokenType.Or ||
      p?.typ == TokenType.Close ||
      p?.typ == TokenType.Eof
    ) {
      return ret;
    }
    return this.parseAndRhs(ret);
  }

  /// Process the right side of the `and`, including chained `and` operators.
  async parseAndRhs(lhs: Constraint): Promise<Constraint> {
    let ret = lhs;
    while (true) {
      const rhs = await this.parseOperand();
      ret = new AndConstraint(ret, rhs);
      const p = await this.peek();
      if (p?.typ == TokenType.And) {
        this.next();
        continue;
      }
      break;
    }
    return ret;
  }
}

/** Defines the type of a particular token. */
enum TokenType {
  And,
  Arg,
  Close,
  Colon,
  Eof,
  Error,
  Not,
  Open,
  Or,
  Predicate
}

/** Represents a single token emitted by the lexer. */
class Token {
  typ: TokenType;
  val: string;

  constructor(typ: TokenType, val: string) {
    this.typ = typ;
    this.val = val;
  }
}

/**
 * A lexical analyzer for the simple query language.
 *
 * Fashioned after that which was presented by Rob Pike in the "Lexical Scanning
 * in Go" talk (https://go.dev/talks/2011/lex.slide). The general idea is that
 * the lexer produces tokens and sends them to a channel, from which a parser
 * would consume them.
 *
 * The design of the lexer involves a finite state machine consisting of
 * function pointers. The starting function determines which function should go
 * next, returning the pointer to that function. This continues until either
 * null is returned by a function, or the end of the input is reached. The
 * "machine" itself is very simple, it continuously invokes the current state
 * function, using its return value as the next function to invoke.
 *
 * As each function processes the input, it may emit one or more tokens. These
 * are sent over a channel from which the recipient, presumably a parser,
 * consumes them. The lexer runs in a separate thread, sending tokens to the
 * channel until either it fills up and blocks, or the input is exhausted.
 */
class QueryLexer {
  input: string;
  // length is the number of characters in the input string
  length: number;
  // start marks the beginning of the current token
  start: number;
  // pos is the index of the next character to be evaluated
  pos: number;
  chan: AsyncQueue<Token>;

  constructor(input: string, chan: AsyncQueue<Token>) {
    this.input = input;
    this.length = input.length;
    this.start = 0;
    this.pos = 0;
    this.chan = chan;
  }

  // emit passes the current token back to the client via the channel.
  async emit(t: TokenType) {
    const text = this.input.slice(this.start, this.pos);
    await this.chan.enqueue(new Token(t, text));
    this.start = this.pos;
  }

  /** passes the message back to the client via the channel */
  async emitError(msg: string) {
    await this.chan.enqueue(new Token(TokenType.Error, msg));
    this.start = this.pos;
  }

  /** passes the given token back to the client via the channel */
  async emitString(t: TokenType, text: string) {
    await this.chan.enqueue(new Token(t, text));
    this.start = this.pos;
  }

  next(): string | undefined {
    if (this.pos >= this.length) {
      return undefined;
    }
    const ch = this.input[this.pos];
    this.pos++;
    return ch;
  }

  /** returns but does not consume the next rune in the input */
  peek(): string | undefined {
    if (this.pos >= this.length) {
      return undefined;
    }
    return this.input[this.pos];
  }

  /** skips over the pending input before this point */
  ignore() {
    this.start = this.pos;
  }

  /** returns `true` if the next rune is from the valid set;
   * the character is not consumed either way. */
  isMatch(valid: string): boolean {
    const ch = this.peek();
    if (ch) {
      return valid.includes(ch);
    } else {
      return false;
    }
  }

  /** consumes the next set of characters if they match
   * the input string, otherwise rewinds and returns `false` */
  acceptString(s: string): boolean {
    for (const r of s) {
      if (this.next() != r) {
        this.rewind();
        return false;
      }
    }
    return true;
  }

  /** consumes a run of runes from the valid set */
  acceptRun(valid: string): boolean {
    const oldPos = this.pos;
    let ch = this.peek();
    while (ch) {
      if (valid.includes(ch)) {
        // consume the character
        this.next();
      } else {
        break;
      }
      ch = this.peek();
    }
    return oldPos < this.pos;
  }

  /** consumes a run of runes until the function returns `false` */
  acceptRunFn(valid: (ch: string) => boolean): boolean {
    const oldPos = this.pos;
    let ch = this.peek();
    while (ch) {
      if (valid(ch)) {
        // consume the character
        this.next();
      } else {
        break;
      }
      ch = this.peek();
    }
    return oldPos < this.pos;
  }

  /** moves the current position back to the start of the current token */
  rewind() {
    this.pos = this.start;
  }
}

// Defining a recursive type definition in TypeScript seems to work if the
// recursive reference is after the other possible variants.
type LexerFun = null | ((l: QueryLexer) => Promise<LexerFun>);

/**
 * Tokenize the input and send tokens to the returned channel.
 *
 * @returns a channel for receiving the tokens.
 */
function lex(input: string): AsyncQueue<Token> {
  const chan = new AsyncQueue<Token>(8);
  // launch the lexer asynchronously to feed tokens to the caller
  setTimeout(async () => {
    const lexer = new QueryLexer(input, chan);
    let fun: LexerFun = lexStart;
    while (fun) {
      fun = await fun(lexer);
    }
    // signal the caller that the lexer is finished
    chan.close();
  }, 0);
  return chan;
}

const WHITESPACE: string = '\t\n\r ';
// operator boundary
const OP_BOUND: string = '\t\n\r (';

/** emits an error token and returns `null' to end lexing */
async function errorf(l: QueryLexer, message: string): Promise<LexerFun> {
  await l.emitError(message);
  return null;
}

async function lexStart(l: QueryLexer): Promise<LexerFun> {
  l.acceptRun(WHITESPACE);
  l.ignore();
  const ch = l.next();
  if (ch) {
    switch (ch) {
      case '(': {
        await l.emit(TokenType.Open);
        return lexStart;
      }
      case ')': {
        await l.emit(TokenType.Close);
        return lexOperator;
      }
      case '-': {
        await l.emit(TokenType.Not);
        return lexStart;
      }
      default: {
        l.rewind();
        return lexPredicate;
      }
    }
  } else {
    await l.emit(TokenType.Eof);
    return null;
  }
}

/** expects to find a boolean operator such as "and" or "or" */
async function lexOperator(l: QueryLexer): Promise<LexerFun> {
  l.acceptRun(WHITESPACE);
  l.ignore();
  switch (l.peek()) {
    case 'a': {
      return lexAnd;
    }
    case 'o': {
      return lexOr;
    }
    default: {
      return lexStart;
    }
  }
}

/** expects to find 'and' followed by whitespace or open paren */
async function lexAnd(l: QueryLexer): Promise<LexerFun> {
  if (l.acceptString('and') && l.isMatch(OP_BOUND)) {
    await l.emit(TokenType.And);
    return lexStart;
  }
  return lexPredicate;
}

/** expects to find 'or' followed by whitespace or open paren */
async function lexOr(l: QueryLexer): Promise<LexerFun> {
  if (l.acceptString('or') && l.isMatch(OP_BOUND)) {
    await l.emit(TokenType.Or);
    return lexStart;
  }
  return lexPredicate;
}

/** expects to read an alphabetic string followed by a colon (:), otherwise an
 * error is emitted */
async function lexPredicate(l: QueryLexer): Promise<LexerFun> {
  l.acceptRunFn(isAlphabetic);
  const ch = l.peek();
  if (ch == ':') {
    await l.emit(TokenType.Predicate);
    l.next();
    await l.emit(TokenType.Colon);
    return lexArgument;
  }
  return errorf(l, 'bare literals unsupported');
}

/** processes double-quoted strings, single-quoted strings,
 * and raw values, including chains of arguments separated by colons */
async function lexArgument(l: QueryLexer): Promise<LexerFun> {
  let ch = l.next();
  if (ch) {
    if (ch === '"') {
      return lexStringDouble;
    } else if (ch === "'") {
      return lexStringSingle;
    }
    // anything else must be a raw value
    l.rewind();
    l.acceptRunFn(is_search_word_rune);
    await l.emit(TokenType.Arg);
    ch = l.peek();
    if (ch === ':') {
      l.next();
      await l.emit(TokenType.Colon);
      return lexArgument;
    }
    return lexOperator;
  }
  // ran out of tokens
  return lexStart;
}

/** expects the current character to be a double-quote
 * and scans the input to find the end of the quoted string */
async function lexStringDouble(l: QueryLexer): Promise<LexerFun> {
  return lexString(l, '"');
}

/** expects the current character to be a single-quote
 * and scans the input to find the end of the quoted string */
async function lexStringSingle(l: QueryLexer): Promise<LexerFun> {
  return lexString(l, "'");
}

/** Scan the quoted string until the end character is found (' or "). */
async function lexString(l: QueryLexer, end: string): Promise<LexerFun> {
  let text = '';
  let ch = l.next();
  while (ch) {
    switch (ch) {
      // pass over escaped characters
      case '\\': {
        ch = l.next();
        if (ch) {
          switch (ch) {
            case '"':
            case "'":
            case ' ':
            case '\t': {
              text += ch;
              break;
            }
            default: {
              // otherwise let replace_escapes() handle it
              text += '\\';
              text += ch;
            }
          }
        } else {
          return errorf(l, 'improperly terminated string');
        }
        break;
      }
      case end: {
        // reached the end of the string
        await l.emitString(TokenType.Arg, text);
        return lexOperator;
      }
      default: {
        text += ch;
      }
    }
    ch = l.next();
  }
  return errorf(l, 'unclosed quoted string');
}

/** return true if the character is classified as a Unicode "Letter" */
function isAlphabetic(ch: string): boolean {
  const regex = /\p{L}/u;
  return ch.match(regex) !== null;
}

/** defines those characters that are part of an unquoted argument, which
 * includes non-whitespace and the symbols supported by the lexer (colon,
 * parentheses) */
function is_search_word_rune(ch: string): boolean {
  if (ch == ':' || ch == '(' || ch == ')') {
    return false;
  }
  return !/\s/.test(ch);
}

export { lex, Token, TokenType, parse, type Constraint };
