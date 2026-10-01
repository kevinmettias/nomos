//! Reading this workspace's Rust as text, because an observer that linked it would be a
//! participant.
//!
//! Eleven modules, and they are one thing: a byte-level reader that knows what every byte
//! of a file is and can therefore find a declaration without a parser. Flat at the crate
//! root they were most of this crate's file count and none of its subject -- a reader
//! looking for what this crate *asserts* had to pick eight claim modules out of nineteen
//! files, the other eleven being the machinery those claims are made with.
//!
//! Declared `pub(crate) mod` rather than re-exported name by name. These modules export a
//! great many small helpers that only each other call, and lifting all of them onto one
//! facade would replace eleven honest paths with one crowded namespace.

pub(crate) mod declaration;
pub(crate) mod functions;
pub(crate) mod masks;
pub(crate) mod module_tree;
pub(crate) mod recogniser;
pub(crate) mod routes;
pub(crate) mod source_files;
pub(crate) mod text;
