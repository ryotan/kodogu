// Reading the root file of a crate at the level the check of the modules needs: its attributes and its items, with
// comments and string literals told apart from code, so that neither can hide or imitate an attribute.

export type Token = {
  readonly kind: 'ident' | 'literal' | 'punct';
  readonly text: string;
  readonly start: number;
  readonly end: number;
};

export type Attribute = {readonly text: string; readonly givesPath: boolean; readonly builtIn: boolean};

export type Item =
  | {readonly kind: 'module'; readonly name: string; readonly text: string}
  | {readonly kind: 'use'; readonly text: string}
  | {readonly kind: 'other'; readonly text: string};

const literalPatterns = [
  // Raw strings, with an optional byte or C prefix: r"…", r#"…"#, br#"…"#.
  /^(?:b|c)?r(#*)"[\s\S]*?"\1/,
  // Strings with escapes, with an optional byte or C prefix.
  /^(?:b|c)?"(?:\\[\s\S]|[^"\\])*"/,
  // Characters and bytes; a quote followed by an identifier and no closing quote is a lifetime, not a literal.
  /^b?'(?:\\[\s\S][^']*|[^'\\])'/,
  /^[0-9][0-9A-Za-z_.]*/,
];
const identifierPattern = /^(?:r#)?[A-Za-z_][A-Za-z0-9_]*/;

// The length of the comment at the start of `rest`, or 0. Block comments nest in Rust.
function commentLength(rest: string): number {
  if (rest.startsWith('//')) {
    const end = rest.indexOf('\n');
    return end < 0 ? rest.length : end;
  }
  if (!rest.startsWith('/*')) {
    return 0;
  }
  let depth = 0;
  for (let index = 0; index < rest.length - 1; index += 1) {
    const pair = rest.slice(index, index + 2);
    if (pair === '/*' || pair === '*/') {
      depth += pair === '/*' ? 1 : -1;
      index += 1;
      if (depth === 0) {
        return index + 1;
      }
    }
  }
  return rest.length;
}

function nextToken(source: string, start: number): Token {
  const rest = source.slice(start);
  for (const pattern of literalPatterns) {
    const match = pattern.exec(rest);
    if (match) {
      return {kind: 'literal', text: match[0], start, end: start + match[0].length};
    }
  }
  const identifier = identifierPattern.exec(rest)?.[0];
  if (identifier !== undefined) {
    return {kind: 'ident', text: identifier, start, end: start + identifier.length};
  }
  return {kind: 'punct', text: rest.charAt(0), start, end: start + 1};
}

export function tokenize(source: string): Token[] {
  const tokens: Token[] = [];
  let index = 0;
  while (index < source.length) {
    const rest = source.slice(index);
    const space = /^\s+/.exec(rest)?.[0].length ?? 0;
    const comment = space > 0 ? 0 : commentLength(rest);
    if (space + comment > 0) {
      index += space + comment;
      continue;
    }
    const token = nextToken(source, index);
    tokens.push(token);
    index = token.end;
  }
  return tokens;
}

const isPunct = (token: Token | undefined, text: string): boolean => token?.kind === 'punct' && token.text === text;

// The index after the `]` that closes the attribute starting at `index`, or `index` when no attribute starts there.
function attributeEnd(tokens: readonly Token[], index: number): number {
  const open = isPunct(tokens[index + 1], '!') ? index + 2 : index + 1;
  if (!isPunct(tokens[index], '#') || !isPunct(tokens[open], '[')) {
    return index;
  }
  let depth = 0;
  for (let at = open; at < tokens.length; at += 1) {
    depth += isPunct(tokens[at], '[') ? 1 : isPunct(tokens[at], ']') ? -1 : 0;
    if (depth === 0) {
      return at + 1;
    }
  }
  return tokens.length;
}

// The name of an identifier as Rust reads it: `r#path` is `path`.
function nameOf(token: Token | undefined): string | undefined {
  return token?.kind === 'ident' ? token.text.replace(/^r#/, '') : undefined;
}

// The index after the group that the opening bracket at `index` starts.
function groupEnd(tokens: readonly Token[], index: number): number {
  let depth = 0;
  for (let at = index; at < tokens.length; at += 1) {
    depth += ['(', '[', '{'].some(open => isPunct(tokens[at], open)) ? 1 : 0;
    depth -= [')', ']', '}'].some(close => isPunct(tokens[at], close)) ? 1 : 0;
    if (depth === 0) {
      return at + 1;
    }
  }
  return tokens.length;
}

// The entries of a parenthesized list, split at its top-level commas.
function entries(tokens: readonly Token[]): Token[][] {
  const result: Token[][] = [[]];
  let index = 0;
  while (index < tokens.length) {
    const token = tokens[index] as Token;
    if (isPunct(token, ',')) {
      result.push([]);
      index += 1;
    } else {
      const end = ['(', '[', '{'].some(open => isPunct(token, open)) ? groupEnd(tokens, index) : index + 1;
      result.at(-1)?.push(...tokens.slice(index, end));
      index = end;
    }
  }
  return result.filter(entry => entry.length > 0);
}

// Built-in attributes that may stand at the root of a crate or on a module. A single name that Rust builds in is never
// resolved to a macro, even when a macro of that name is imported; a path such as `a::b` may be a macro.
const builtInAttributes = new Set([
  'allow',
  'cfg',
  'cfg_attr',
  'deny',
  'deprecated',
  'doc',
  'expect',
  'forbid',
  'macro_use',
  'no_implicit_prelude',
  'no_std',
  'path',
  'recursion_limit',
  'type_length_limit',
  'warn',
]);

// The path that names an attribute, from the start of its input: `path`, `r#path` read as `path`, or `a::b`.
function attributeName(input: readonly Token[]): string {
  const segments: string[] = [];
  let index = 0;
  while (nameOf(input[index]) !== undefined) {
    segments.push(nameOf(input[index]) as string);
    if (!isPunct(input[index + 1], ':') || !isPunct(input[index + 2], ':')) {
      break;
    }
    index += 3;
  }
  return segments.join('::');
}

// The names of the attributes that the input of an attribute, the tokens inside `#[…]`, applies: its own name, or for
// `cfg_attr` the names of the attributes after its predicate, at any depth. The input of any other attribute, such as
// the text of `doc`, is not read.
function appliedNames(input: readonly Token[]): string[] {
  const name = attributeName(input);
  if (name !== 'cfg_attr' || !isPunct(input[1], '(')) {
    return [name];
  }
  return entries(input.slice(2, groupEnd(input, 1) - 1))
    .slice(1)
    .flatMap(appliedNames);
}

function textOf(source: string, tokens: readonly Token[]): string {
  const first = tokens[0];
  const last = tokens.at(-1);
  return first && last ? source.slice(first.start, last.end).replace(/\s+/g, ' ') : '';
}

function splitAttributes(source: string, tokens: readonly Token[]): {attributes: Attribute[]; rest: Token[]} {
  const attributes: Attribute[] = [];
  const rest: Token[] = [];
  let index = 0;
  while (index < tokens.length) {
    const end = attributeEnd(tokens, index);
    if (end > index) {
      const attribute = tokens.slice(index, end);
      const input = attribute.slice(isPunct(attribute[1], '!') ? 3 : 2, -1);
      const names = appliedNames(input);
      attributes.push({
        text: textOf(source, attribute),
        givesPath: names.includes('path'),
        builtIn: names.every(name => builtInAttributes.has(name)),
      });
      index = end;
    } else {
      rest.push(tokens[index] as Token);
      index += 1;
    }
  }
  return {attributes, rest};
}

// The tokens after a visibility such as `pub` or `pub(crate)`.
function withoutVisibility(tokens: readonly Token[]): readonly Token[] {
  if (tokens[0]?.text !== 'pub') {
    return tokens;
  }
  if (!isPunct(tokens[1], '(')) {
    return tokens.slice(1);
  }
  const close = tokens.findIndex(token => isPunct(token, ')'));
  return close < 0 ? [] : tokens.slice(close + 1);
}

function classify(source: string, tokens: readonly Token[]): Item {
  const text = textOf(source, tokens);
  const body = withoutVisibility(tokens);
  const [keyword, name] = body;
  if (keyword?.text === 'mod' && name?.kind === 'ident' && body.length === 2) {
    return {kind: 'module', name: name.text, text};
  }
  return keyword?.text === 'use' ? {kind: 'use', text} : {kind: 'other', text};
}

// The attributes and the items of a root at brace depth zero. An item ends at `;`, or at the `}` that closes its
// braces unless it is a `use` item, whose braces group imported names.
export function readRoot(source: string): {attributes: Attribute[]; items: Item[]} {
  const {attributes, rest} = splitAttributes(source, tokenize(source));
  const items: Item[] = [];
  let current: Token[] = [];
  let depth = 0;
  for (const token of rest) {
    depth += isPunct(token, '{') ? 1 : isPunct(token, '}') ? -1 : 0;
    const ends =
      depth === 0 && (isPunct(token, ';') || (isPunct(token, '}') && withoutVisibility(current)[0]?.text !== 'use'));
    if (ends && isPunct(token, ';')) {
      items.push(classify(source, current));
      current = [];
    } else if (ends) {
      items.push(classify(source, [...current, token]));
      current = [];
    } else {
      current.push(token);
    }
  }
  return {attributes, items: current.length > 0 ? [...items, classify(source, current)] : items};
}
