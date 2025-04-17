use crate::{stelaro_ast::ast::{NodeId, Stelo}, stelaro_common::{def::DefKind, DefId}, stelaro_context::TyCtxt, stelaro_ty::ty::PrimTy};


struct Resolver<'tcx> {
    tcx: TyCtxt<'tcx>,
    next_node_id: NodeId,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Res<Id = NodeId> {
    /// 定義 (e.g., function, struct, module)
    /// `DefKind` は定義の種類を表す
    /// `DefId` は一意に定義を識別できる
    Def(DefKind, DefId),

    /// ローカル変数 (let文, 関数のパラメータ)
    /// `Id` はローカル変数が宣言された場所を表す (e.g., the NodeId of the Let statement or pattern).
    Local(Id),

    /// プリミティブ型 (e.g., `i32`, `bool`).
    PrimTy(PrimTy),

    /// 名前解決に失敗したとき
    Err,
}

impl<'tcx> Resolver<'tcx> {
    fn next_node_id(&mut self) -> NodeId {
        let start = self.next_node_id;
        let next = NodeId::from_u32(start.as_u32() + 1);
        self.next_node_id = next;
        start
    }

    pub fn resolve_stelo(&mut self, stelo: &Stelo) {
        
    }
}