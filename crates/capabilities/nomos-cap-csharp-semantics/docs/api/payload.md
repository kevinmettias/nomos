# API — payload

## Contract

One `selection` line naming the build, then one `symbol` line per symbol that build defines,
one `define` or `undef` line per file-level definition directive, and one `region` line per
branch, each tab-separated:

```text
selection	src/App/App.csproj	Debug	net8.0
symbol	DEBUG
define	1	LOCAL
region	if	3	5	skipped	DEBUG && !LOCAL
region	else	5	7	compiled
```

A region line's condition is its last field, collapsed to single spaces, and empty for `#else`.

## Arguments — `Parse_Payload`

A payload with no region line is valid: a file with no conditional directive answers with its
selection and nothing else. A payload with no selection line is not, because an answer about no
build is not an answer about any.

## Errors — `Parse_Payload`

Returns `PayloadRefusal` when the bytes are not UTF-8; when a line is none of the four kinds;
when the selection line is missing, repeated, or not three fields; when a region line is not
five fields or names a branch or state this schema does not define; or when a line number is not
a count.
