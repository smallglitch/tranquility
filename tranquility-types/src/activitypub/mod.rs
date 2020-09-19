use serde_json::{json, Value};

pub fn context_field() -> Value {
    json!(["https://www.w3.org/ns/activitystreams"])
}

pub mod activity;
pub mod actor;
pub mod attachment;
pub mod object;
pub mod tag;

pub use activity::Activity;
pub use actor::{Actor, PublicKey};
pub use attachment::Attachment;
pub use object::Object;
pub use tag::Tag;
