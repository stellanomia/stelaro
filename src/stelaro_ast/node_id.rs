#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);
pub const STELO_NODE_ID: NodeId = NodeId(0);

impl NodeId {

    pub fn from_u32(value: u32) -> Self {
        NodeId(value)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
