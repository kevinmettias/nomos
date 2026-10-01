# API — payload

## Contract

Ten `descriptor` lines, one per `MET-006` property, then one `function` line per function the
file defines, each tab-separated:

```text
descriptor	aggregation	not-aggregable
function	Walker::Step	9	7
```

`Complexity_Descriptor` is the one descriptor this schema's values are read under. It is a
function rather than a constant so its text is written once and read by the provider that
encodes it and the tests that pin it.

## Arguments — `Parse_Payload`

An empty byte string is **not** a valid payload, unlike `nomos.controlflow.reachability.v1`'s:
every answer carries its descriptor, so a file defining no function answers with ten
descriptor lines and no function line. That keeps "this file defines none" and "nobody
measured this file" different facts.

## Errors — `Parse_Payload`

Returns `PayloadRefusal` when the bytes are not UTF-8; when a line is neither a descriptor nor
a function; when a descriptor line names a property `MET-006` does not, names one twice, or
carries an aggregation or directionality label this schema does not define; when any of the
ten properties has no line at all; or when a function line does not have exactly a name, a line
number and a count.
