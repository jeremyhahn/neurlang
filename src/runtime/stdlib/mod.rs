//! Standard Library Extensions for Neurlang
//!
//! Provides common data structures and utilities as EXT.CALL extensions.
//!
//! # Architecture
//!
//! Stdlib uses a handle-based approach:
//! - `vec_new()` returns a handle (u64)
//! - `vec_push(handle, value)` operates on that handle
//! - `vec_free(handle)` releases the resource
//!
//! This allows IR code to work with complex data structures using only
//! register values (u64).

pub mod collections;
pub mod http;
pub mod json;
pub mod strings;

use super::ExtensionRegistry;

/// Register all stdlib extensions
pub fn register_stdlib(registry: &mut ExtensionRegistry) {
    collections::register_vec_extensions(registry);
    collections::register_hashmap_extensions(registry);
    strings::register_string_extensions(registry);
    json::register_json_extensions(registry);
    http::register_http_extensions(registry);
}

/// Extension IDs for stdlib (starting at 100 to avoid conflicts with crypto)
pub mod ext_ids {
    // Vec operations (100-119)
    pub const VEC_NEW: u32 = 100;
    pub const VEC_WITH_CAPACITY: u32 = 101;
    pub const VEC_PUSH: u32 = 102;
    pub const VEC_POP: u32 = 103;
    pub const VEC_GET: u32 = 104;
    pub const VEC_SET: u32 = 105;
    pub const VEC_LEN: u32 = 106;
    pub const VEC_CAPACITY: u32 = 107;
    pub const VEC_CLEAR: u32 = 108;
    pub const VEC_FREE: u32 = 109;
    pub const VEC_EXTEND: u32 = 110;
    pub const VEC_INSERT: u32 = 111;
    pub const VEC_REMOVE: u32 = 112;

    // HashMap operations (120-139)
    pub const HASHMAP_NEW: u32 = 120;
    pub const HASHMAP_INSERT: u32 = 121;
    pub const HASHMAP_GET: u32 = 122;
    pub const HASHMAP_REMOVE: u32 = 123;
    pub const HASHMAP_CONTAINS: u32 = 124;
    pub const HASHMAP_LEN: u32 = 125;
    pub const HASHMAP_CLEAR: u32 = 126;
    pub const HASHMAP_FREE: u32 = 127;
    pub const HASHMAP_KEYS: u32 = 128;
    pub const HASHMAP_VALUES: u32 = 129;

    // String operations (140-169)
    pub const STRING_NEW: u32 = 140;
    pub const STRING_FROM_BYTES: u32 = 141;
    pub const STRING_LEN: u32 = 142;
    pub const STRING_CONCAT: u32 = 143;
    pub const STRING_SUBSTR: u32 = 144;
    pub const STRING_FIND: u32 = 145;
    pub const STRING_REPLACE: u32 = 146;
    pub const STRING_SPLIT: u32 = 147;
    pub const STRING_TRIM: u32 = 148;
    pub const STRING_TO_UPPER: u32 = 149;
    pub const STRING_TO_LOWER: u32 = 150;
    pub const STRING_STARTS_WITH: u32 = 151;
    pub const STRING_ENDS_WITH: u32 = 152;
    pub const STRING_TO_BYTES: u32 = 153;
    pub const STRING_FREE: u32 = 154;
    pub const STRING_PARSE_INT: u32 = 155;
    pub const STRING_PARSE_FLOAT: u32 = 156;
    pub const STRING_FROM_INT: u32 = 157;
    pub const STRING_FROM_FLOAT: u32 = 158;

    // JSON operations (170-189)
    pub const JSON_PARSE: u32 = 170;
    pub const JSON_STRINGIFY: u32 = 171;
    pub const JSON_GET: u32 = 172;
    pub const JSON_SET: u32 = 173;
    pub const JSON_GET_TYPE: u32 = 174;
    pub const JSON_ARRAY_LEN: u32 = 175;
    pub const JSON_ARRAY_GET: u32 = 176;
    pub const JSON_ARRAY_PUSH: u32 = 177;
    pub const JSON_OBJECT_KEYS: u32 = 178;
    pub const JSON_FREE: u32 = 179;
    pub const JSON_NEW_OBJECT: u32 = 180;
    pub const JSON_NEW_ARRAY: u32 = 181;

    // HTTP operations (190-209)
    pub const HTTP_GET: u32 = 190;
    pub const HTTP_POST: u32 = 191;
    pub const HTTP_PUT: u32 = 192;
    pub const HTTP_DELETE: u32 = 193;
    pub const HTTP_RESPONSE_STATUS: u32 = 194;
    pub const HTTP_RESPONSE_BODY: u32 = 195;
    pub const HTTP_RESPONSE_FREE: u32 = 196;
    pub const HTTP_GET_WITH_HEADERS: u32 = 197;
    pub const HTTP_POST_WITH_HEADERS: u32 = 198;
}
