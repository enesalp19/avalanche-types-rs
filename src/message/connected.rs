use crate::{ids, version};

/// ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/version#Application
/// ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/message#InternalMsgBuilder
/// ref. "InternalConnected"
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub struct Message {
    pub version: version::ApplicationVersion,

    /// The node ID that this node is connected to.
    pub node_id: ids::node::Id,
}

/// ref. https://doc.rust-lang.org/std/string/trait.ToString.html
/// ref. https://doc.rust-lang.org/std/fmt/trait.Display.html
/// Use "Self.to_string()" to directly invoke this
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "msg connected to node {} with version {:?}",
            self.node_id, self.version
        )
    }
}