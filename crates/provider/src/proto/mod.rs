//! Protobuf types generated from the TRON protocol `.proto` files via tonic_prost_build.
//!
//! These types are intentionally **not** re-exported: callers work with the
//! public domain types in [`crate::types`]. The codec conversions
//! (domain ↔ proto) live alongside the transport implementations.

// The generated code contains many dead_code structs (full proto schema) and
// enum variants whose names share a common suffix — none of that is our code
// to fix, so suppress all lints for this module.
#[allow(dead_code, unused_imports, clippy::all, clippy::pedantic)]
mod inner {
    tonic::include_proto!("protocol");
}
pub(crate) use inner::*;
