
stelaro_macros::newtype_index! {
   /// AST ノードを識別する。
    #[orderable]
    #[debug_format = "NodeId({})"]
    pub struct NodeId {
        /// ステロのルートを表すのに使われる。
        const STELO_NODE_ID = 0;
    }
}
