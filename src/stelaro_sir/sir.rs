use std::{collections::{BTreeMap, HashMap}, fmt};

use crate::stelaro_common::{IndexVec, LocalDefId, Span};
use crate::stelaro_sir::sir_id::{ItemLocalId, SirId};


#[derive(Copy, Clone, Debug)]
pub enum OwnerNode<'sir> {
    Item(&'sir Item<'sir>),
    Stelo(&'sir Mod<'sir>),
}

/// SIRノードと、同じSIRオーナー内におけるその親のIDを結びつけたもの。
///
/// ノード自体がSIRオーナーである場合、親のIDは無意味である。
#[derive(Clone, Copy, Debug)]
pub struct ParentedNode<'tcx> {
    pub parent: ItemLocalId,
    pub node: Node<'tcx>,
}

/// 現在のオーナーの内部にあるすべてのSIRノードのマップ。
/// これらのノードは `ItemLocalId`` をキーに、親ノードのインデックスとともにマッピングされる。
pub struct OwnerNodes<'tcx> {
    /// 「現在のオーナーに対応する完全なSIR。
    // 0番目のノードの親は `ItemLocalId::INVALID` に設定されており、アクセスできない
    pub nodes: IndexVec<ItemLocalId, ParentedNode<'tcx>>,
    /// ローカルな Body の内容
    pub bodies: BTreeMap<ItemLocalId, &'tcx Body<'tcx>>,
}

impl<'tcx> OwnerNodes<'tcx> {
    pub fn node(&self) -> OwnerNode<'tcx> {
        // Indexing では OwnerNode であることを保証しなければならない
        self.nodes[ItemLocalId::ZERO].node.as_owner().unwrap()
    }
}


impl fmt::Debug for OwnerNodes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnerNodes")
            // すべてのノードへのポインタをすべて出力すると、可読性が低下するため行わない。
            .field("node", &self.nodes[ItemLocalId::ZERO])
            .field(
                "parents",
                &fmt::from_fn(|f| {
                    f.debug_list()
                        .entries(self.nodes.iter_enumerated().map(|(id, parented_node)| {
                            fmt::from_fn(move |f| write!(f, "({id:?}, {:?})", parented_node.parent))
                        }))
                        .finish()
                }),
            )
            .field("bodies", &self.bodies)
            .finish()
    }
}

/// ASTノードを Lowering した結果、得られる完全な情報。
#[derive(Debug)]
pub struct OwnerInfo<'sir> {
    /// SIRの内容
    pub nodes: OwnerNodes<'sir>,

    /// 各ネストされたオーナーから、その親のローカルIDへのマップ
    pub parenting: HashMap<LocalDefId, ItemLocalId>,
}

#[derive(Copy, Clone, Debug)]
pub enum MaybeOwner<'tcx> {
    Owner(&'tcx OwnerInfo<'tcx>),
    NonOwner(SirId),
    /// 使われていない `LocalDefId` のためのプレースホルダとして使われる
    Phantom,
}

impl<'tcx> MaybeOwner<'tcx> {
    pub fn as_owner(self) -> Option<&'tcx OwnerInfo<'tcx>> {
        match self {
            MaybeOwner::Owner(i) => Some(i),
            MaybeOwner::NonOwner(_) | MaybeOwner::Phantom => None,
        }
    }

    pub fn unwrap(self) -> &'tcx OwnerInfo<'tcx> {
        self.as_owner().unwrap_or_else(|| panic!("SIR の所有者ではありません"))
    }
}

#[derive(Debug)]
pub struct Crate<'sir> {
    pub owners: IndexVec<LocalDefId, MaybeOwner<'sir>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Body<'sir> {
    pub params: &'sir [Param<'sir>],
    pub value: &'sir Expr<'sir>,
}

impl<'sir> Body<'sir> {
    pub fn id(&self) -> BodyId {
        BodyId { sir_id: self.value.sir_id }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Node<'sir> {
    Param(&'sir Param<'sir>),
    Item(&'sir Item<'sir>),
    Expr(&'sir Expr<'sir>),
    Stmt(&'sir Stmt<'sir>),
    PathSegment(&'sir PathSegment<'sir>),
    Ty(&'sir Ty<'sir>),
    Pat(&'sir Pat<'sir>),
    Block(&'sir Block<'sir>),
    LetStmt(&'sir LetStmt<'sir>),
    Stelo(&'sir Mod<'sir>),
    Err(Span),
}

impl<'sir> Node<'sir> {
    pub fn as_owner(self) -> Option<OwnerNode<'sir>> {
        match self {
            Node::Item(i) => Some(OwnerNode::Item(i)),
            Node::Stelo(i) => Some(OwnerNode::Stelo(i)),
            _ => None,
        }
    }
}