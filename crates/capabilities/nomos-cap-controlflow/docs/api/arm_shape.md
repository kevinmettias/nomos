# API — arm_shape

## Contract

`ArmShape` is deliberately closed: a shape it cannot name is a shape a `Syntactic`-level
provider does not flag, by construction. Extending recognition to a new shape means adding
a new variant here, not widening what "obvious" means through a default arm.

## Arguments — `Parse_Payload`

An empty byte string is a valid `bytes` argument — it decodes to a payload with no flagged
sites at all. This is unlike [`nomos_cap_dependency::Parse_Payload`], which always expects
a leading `package` line; this schema has no such header line to require.

## Errors — `Parse_Payload`

Returns [`PayloadRefusal`] when the bytes are not valid UTF-8, or when a line does not have
exactly the four tab-separated fields (`site`, function, binding, shape) this schema
declares.
