use std::fmt;

use crate::stelaro_common::DelayedMap;
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir_typecheck::infer::InferCtxt;
use crate::stelaro_ty::Ty;
use crate::stelaro_ty::fold::{FallibleTypeFolder, TypeFoldable, TypeFolder, TypeSuperFoldable};
use crate::stelaro_ty::ty::TyVid;
use crate::stelaro_ty::visit::{Flags, TypeFlags};

/// `fully_resolve` が失敗したときに返されるエラー。
/// 型推論が完了した後も、解決されずに残った型変数が存在することを示す。
#[derive(Copy, Clone, Debug)]
pub struct UnresolvedInferVar {
    /// 解決できなかった型変数のID。
    vid: TyVid,
}

impl UnresolvedInferVar {
    pub fn new(vid: TyVid) -> Self {
        Self { vid }
    }
}

impl fmt::Display for UnresolvedInferVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "型を推論できませんでした。型注釈を追加する必要があります。 (未解決の変数: {:?})", self.vid)
    }
}

/// 型変数を、現時点で判明している情報で自分勝手に (opportunistically) 解決するフォルダー。
///
/// このフォルダーはいつでも安全に使用できます。型変数が既に何らかの型に統合されていれば、
/// その型に深く (再帰的に) 置き換えます。もし変数がまだ解決されていなければ、
/// その変数のルート（代表となる変数）に置き換えます。
/// この操作は決して失敗しません。
pub struct OpportunisticVarResolver<'a, 'tcx> {
    infcx: &'a InferCtxt<'tcx>,
    /// フォルダー自身は可変な状態を持たないため (infcxの状態は変化させない)、
    /// 計算結果をキャッシュすることが安全にできます。
    /// `DelayedMap` を使うことで、小さな型に対するキャッシュのオーバーヘッドを回避し、
    /// 巨大な型の場合にのみキャッシュを有効にするという最適化を行っています。
    cache: DelayedMap<Ty<'tcx>, Ty<'tcx>>,
}

impl<'a, 'tcx> OpportunisticVarResolver<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtxt<'tcx>) -> Self {
        OpportunisticVarResolver {
            infcx,
            cache: DelayedMap::default(),
        }
    }
}


impl<'a, 'tcx> TypeFolder<'tcx> for OpportunisticVarResolver<'a, 'tcx> {
    fn tcx(&self) -> crate::stelaro_context::TyCtxt<'tcx> {
        self.infcx.tcx
    }

    #[inline]
    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        if let Some(ty) = self.cache.get(&t) {
            *ty
        } else {
            let shallow = self.infcx.shallow_resolve(t);
            let res = shallow.super_fold_with(self);
            assert!(self.cache.insert(t, res));
            res
        }
    }
}

/// `fully_resolve` は、すべての型変数それらの具体的な結果で置き換えます。
/// もし、いずれかの変数が置き換えられない場合（一度も unify されなかったなど）、
/// `Err` の結果が返されます。
pub fn fully_resolve<'tcx, T>(infcx: &InferCtxt<'tcx>, value: T) -> Result<T, UnresolvedInferVar>
where
    T: TypeFoldable<'tcx>,
{
    value.try_fold_with(&mut FullTypeResolver { infcx })
}

/// すべての型変数を完全に解決しようと試みるフォルダー。
/// 未解決の変数が残っていた場合、`Err`を返す。
pub struct FullTypeResolver<'a, 'tcx> {
    infcx: &'a InferCtxt<'tcx>,
}

impl<'a, 'tcx> FallibleTypeFolder<'tcx> for FullTypeResolver<'a, 'tcx> {
    type Error = UnresolvedInferVar;

    fn tcx(&self) -> TyCtxt<'tcx> {
        self.infcx.tcx
    }

    fn try_fold_ty(&mut self, ty: Ty<'tcx>) -> Result<Ty<'tcx>, Self::Error> {
        if !ty.flags().contains(TypeFlags::HAS_TY_INFER) {
            return Ok(ty);
        }

        todo!()
    }
}
