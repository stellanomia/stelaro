use std::{collections::HashMap, fmt};

use crate::stelaro_ast::ast::{BinOp, UnOp};
use crate::stelaro_diagnostics::ErrorEmitted;
use crate::stelaro_common::{sym, Ident, IndexVec, LocalDefId, Span, SortedMap, Spanned, Symbol};
use crate::stelaro_sir::{def::Res, sir_id::{OwnerId, ItemLocalId, SirId, STELO_SIR_ID}};
use crate::stelaro_ty::ty::{FloatTy, IntTy, UintTy};


#[derive(Copy, Clone, Debug)]
pub enum OwnerNode<'sir> {
    Item(&'sir Item<'sir>),
    Stelo(&'sir Mod<'sir>),
}

impl<'sir> OwnerNode<'sir> {
    pub fn span(&self) -> Span {
        match self {
            OwnerNode::Item(Item { span, .. }) => *span,
            OwnerNode::Stelo(Mod { spans: ModSpan { inner_span, .. }, .. }) => *inner_span,
        }
    }

    pub fn def_id(self) -> OwnerId {
        match self {
            OwnerNode::Item(Item { owner_id, .. }) => *owner_id,
            OwnerNode::Stelo(..) => STELO_SIR_ID.owner,
        }
    }
}

impl<'sir> From<OwnerNode<'sir>> for Node<'sir> {
    fn from(val: OwnerNode<'sir>) -> Self {
        match val {
            OwnerNode::Item(n) => Node::Item(n),
            OwnerNode::Stelo(n) => Node::Stelo(n),
        }
    }
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
/// これらのノードは `ItemLocalId` をキーに、親ノードのインデックスとともにマッピングされる。
pub struct OwnerNodes<'tcx> {
    /// 「現在のオーナーに対応する完全なSIR。
    // 0番目のノードの親は `ItemLocalId::INVALID` に設定されており、アクセスできない
    pub nodes: IndexVec<ItemLocalId, ParentedNode<'tcx>>,
    /// ローカルな Body の内容
    pub bodies: SortedMap<ItemLocalId, &'tcx Body<'tcx>>,
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
pub struct Stelo<'sir> {
    pub owners: IndexVec<LocalDefId, MaybeOwner<'sir>>,
}


#[derive(Debug, Clone, Copy)]
pub struct Path<'sir, R = Res<SirId>> {
    pub span: Span,
    /// パスの解決結果。
    pub res: R,
    /// パス内のセグメント。`::` によって区切られたものです。
    pub segments: &'sir [PathSegment],
}


/// パスのセグメント。識別子や型の集合から構成されます。
#[derive(Debug, Clone, Copy)]
pub struct PathSegment {
    /// このパスセグメントの識別子部分。
    pub ident: Ident,
    pub sir_id: SirId,
    pub res: Res,
}

/// `{ .. }` で表される文のブロック。
#[derive(Debug, Clone, Copy)]
pub struct Block<'sir> {
    /// ブロック内の文。
    pub stmts: &'sir [Stmt<'sir>],
    /// ブロックの末尾にある、セミコロンが付かない式（存在する場合）。
    pub expr: Option<&'sir Expr<'sir>>,
    pub sir_id: SirId,
    /// スパンには、ブロックを囲む波括弧 `{` と `}` が含まれます。
    pub span: Span,
}

impl<'sir> Block<'sir> {
    pub fn innermost_block(&self) -> &Block<'sir> {
        let mut block = self;
        while let Some(Expr { kind: ExprKind::Block(inner_block), .. }) = block.expr {
            block = inner_block;
        }
        block
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Pat<'sir> {
    pub sir_id: SirId,
    pub kind: PatKind<'sir>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum PatKind<'sir> {
    /// `_` のような、ワイルドカードのパターンを表す。
    WildCard,

    /// 新しい束縛を表します。
    /// `SirId` は、束縛される変数の正規のIDです。
    Binding(SirId, Ident, Option<&'sir Pat<'sir>>),
}

#[derive(Debug, Clone, Copy)]
pub struct Stmt<'sir> {
    pub sir_id: SirId,
    pub kind: StmtKind<'sir>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum StmtKind<'sir> {
    /// let文
    Let(&'sir LetStmt<'sir>),

    /// アイテムへのバインディング
    Item(ItemId),

    /// 末尾にセミコロンが付かない式
    Expr(&'sir Expr<'sir>),

    /// 末尾にセミコロンが付く式
    Semi(&'sir Expr<'sir>),

    /// Return文
    Return(&'sir Expr<'sir>),

    /// While文
    While(&'sir Expr<'sir>, &'sir Block<'sir>)
}

/// `let` 文を表す (i.e., `let <pat>:<ty> = <init>;`).
#[derive(Debug, Clone, Copy)]
pub struct LetStmt<'sir> {
    pub pat: &'sir Pat<'sir>,
    /// 型アテノーション
    pub ty: Option<&'sir Ty<'sir>>,
    /// 値を設定するための初期化式（存在する場合）。
    pub init: Option<&'sir Expr<'sir>>,
    pub sir_id: SirId,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BodyId {
    pub sir_id: SirId,
}


/// 関数の本体。
/// 本体には、関数本体(という式)そのものだけでなく、引数のパターンも含まれます。
/// なぜなら、それらは呼び出し元が実際には関知しないものであるためです。
///
/// fn foo(x: i32, y: i32) -> i32 {
///     x + y
/// }
///
/// ここで、`foo()` に関連付けられた `Body` は以下のものを含みます:
///
/// - `x`, 'y' パターンを含む `params` 配列
/// - `x + y` 式を含む `value`
///
/// すべての本体は **owner** (所有者) を持ちます。これは
/// `body_owner_def_id()` を使って SIR マップ経由でアクセスできます。
#[derive(Debug, Clone, Copy)]
pub struct Body<'sir> {
    pub params: &'sir [Param],
    pub value: &'sir Expr<'sir>,
}

impl<'sir> Body<'sir> {
    pub fn id(&self) -> BodyId {
        BodyId { sir_id: self.value.sir_id }
    }
}

pub type Lit = Spanned<LitKind>;

#[derive(Debug, Clone, Copy)]
pub struct Expr<'sir> {
    pub sir_id: SirId,
    pub kind: ExprKind<'sir>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprKind<'sir> {
    /// 関数呼び出し。
    ///
    /// 最初のフィールドは関数自身 (通常は `ExprKind::Path`) に解決され、
    /// 2番目のフィールドは引数のリストです。
    Call(&'sir Expr<'sir>, &'sir [Expr<'sir>]),

    /// 二項演算 (e.g., `a + b`, `a * b`).
    Binary(BinOp, &'sir Expr<'sir>, &'sir Expr<'sir>),

    /// 一項演算 (e.g., `!x`, `-x`).
    Unary(UnOp, &'sir Expr<'sir>),

    /// リテラル (e.g., `1`, `"foo"`).
    Lit(&'sir Lit),

    /// `if` ブロック。 else ブロックを持つことがあります。
    ///
    /// すなわち、`if <expr> { <expr> } else { <expr> }`。
    ///
    /// "then" 節の式は常に `ExprKind::Block` です。
    /// "else" 節の式が存在する場合、常に `ExprKind::Block` (`else` の場合)
    /// または `ExprKind::If` (`else if` の場合) になります。
    If(&'sir Expr<'sir>, &'sir Expr<'sir>, Option<&'sir Expr<'sir>>),

    Path(Path<'sir>),

    /// ブロック式
    Block(&'sir Block<'sir>),

    /// 代入 (e.g., `a = foo()`)
    Assign(&'sir Expr<'sir>, &'sir Expr<'sir>, Span),

    /// `return expr;` を表す
    Ret(Option<&'sir Expr<'sir>>),

    Err(ErrorEmitted),
}

// アイテムの本体は、`Stelo` 内の別の
// ハッシュマップに格納されます。ここでは、後で取得できるように
// アイテムの sir-id を記録するだけです。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId {
    pub owner_id: OwnerId,
}

impl ItemId {
    #[inline]
    pub fn sir_id(&self) -> SirId {
        // アイテムは常に SIR の owner です。
        SirId::make_owner(self.owner_id.def_id)
    }
}

/// アイテム
#[derive(Debug, Clone, Copy)]
pub struct Item<'sir> {
    pub owner_id: OwnerId,
    pub kind: ItemKind<'sir>,
    pub span: Span,
}


impl<'sir> Item<'sir> {
    #[inline]
    pub fn sir_id(&self) -> SirId {
        // アイテムは常に sir の owner です。
        SirId::make_owner(self.owner_id.def_id)
    }

    pub fn item_id(&self) -> ItemId {
        ItemId { owner_id: self.owner_id }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ItemKind<'sir> {
    /// 関数定義
    Fn {
        sig: FnSig<'sir>,
        ident: Ident,
        body: BodyId,
    },

    /// モジュール
    Mod(Ident, &'sir Mod<'sir>),
}

/// SIR での型を表す。
#[derive(Debug, Clone, Copy)]
pub struct Ty<'sir> {
    pub sir_id: SirId,
    pub span: Span,
    pub kind: TyKind<'sir>,
}

#[derive(Debug, Clone, Copy)]
pub enum TyKind<'sir> {
    Path(Path<'sir>),

    // () 型。ボトム型として機能する
    // タプルが実装できた際に、これを削除し空のTupleがUnitを表すように変更する
    Unit,

    Infer,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimTy {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
}

impl PrimTy {
    pub fn from_name(name: Symbol) -> Option<PrimTy> {
        let ty = match name {
            sym::BOOL => PrimTy::Bool,
            sym::CHAR => PrimTy::Char,
            sym::I32 => PrimTy::Int(IntTy::I32),
            sym::I64 => PrimTy::Int(IntTy::I64),
            _ => return None,
        };

        Some(ty)
    }
}

/// 関数のシグネチャを表します。
#[derive(Debug, Clone, Copy)]
pub struct FnSig<'sir> {
    // pub header: FnHeader,
    pub decl: &'sir FnDecl<'sir>,
    pub span: Span,
}

/// 関数のヘッダーにおけるパラメーターを表します。
#[derive(Debug, Clone, Copy)]
pub struct Param {
    pub sir_id: SirId,
    pub ident: Ident,
    pub ty_span: Span,
    pub span: Span,
}

/// 関数宣言のヘッダー（本体ではない）を表します。
#[derive(Debug, Clone, Copy)]
pub struct FnDecl<'sir> {
    /// 関数の仮引数の型。
    ///
    /// 追加の引数データは、関数の[本体](Body::params)に格納されます。
    pub inputs: &'sir [Ty<'sir>],
    pub output: FnRetTy<'sir>,
}

#[derive(Debug, Clone, Copy)]
pub enum FnRetTy<'sir> {
    /// 戻り値の型が指定されていない。
    ///
    /// 関数の場合は `()` がデフォルトとなり、
    /// クロージャの場合は型推論がデフォルトとなる。Spanは、戻り値の型が
    /// 挿入されるであろう場所を指す。
    DefaultReturn(Span),
    /// それ以外すべて。
    Return(&'sir Ty<'sir>),
}

impl<'sir> FnRetTy<'sir> {
    #[inline]
    pub fn span(&self) -> Span {
        match *self {
            Self::DefaultReturn(span) => span,
            Self::Return(ty) => ty.span,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mod<'sir> {
    pub spans: ModSpan,
    pub item_ids: &'sir [ItemId],
}

#[derive(Debug, Clone, Copy)]
pub struct ModSpan {
    /// `{` の直後の最初のトークンから、`}` の直前の最後のトークンまでのスパン。
    pub inner_span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LitKind {
    /// 文字列リテラル (`"foo"`)。シンボルはアンエスケープ (エスケープ解除) されているため、
    /// 元のトークンのシンボルとは異なる場合があります。
    Str(Symbol),
    /// 文字リテラル (`'a'`)
    Char(char),
    /// 整数リテラル (`1`)
    Int(u128),
    /// 浮動小数点リテラル。
    /// `LitKind` が `Eq` と `Hash` を実装できるように、
    /// `f64` ではなくシンボルとして格納されます。
    Float(Symbol),
    /// ブールリテラル (`true`, `false`)。
    Bool(bool),
    /// 何らかの点で整形式でなかったリテラルのためのプレースホルダー。
    Err(ErrorEmitted),
}


#[derive(Copy, Clone, Debug)]
pub enum Node<'sir> {
    Param(&'sir Param),
    Item(&'sir Item<'sir>),
    Expr(&'sir Expr<'sir>),
    Stmt(&'sir Stmt<'sir>),
    PathSegment(&'sir PathSegment),
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
