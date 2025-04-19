use std::array::IntoIter;

use crate::stelaro_ast::ast::NodeId;
use crate::stelaro_common::DefId;
use crate::stelaro_ty::ty::PrimTy;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DefKind {
    // Mod,    // モジュール
    // Struct,
    // Enum,
    // Field,
    Fn, // 関数定義
    // Static, // Static item
    // Const,  // Const item
    Let,    // Let文
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Res<Id = NodeId> {
    /// 定義 (e.g., function, struct, module)
    /// `DefKind` は定義の種類を表す
    /// `DefId` は一意に定義を識別できる
    Def(DefKind, DefId),

    /// ローカル変数 (let文, 関数のパラメータ)
    /// `Id` はローカル変数が宣言された場所を表す (let文やパターンに対する NodeId など)
    Local(Id),

    /// プリミティブ型 (e.g., `i32`, `bool`)
    PrimTy(PrimTy),

    /// 名前解決に失敗したとき
    Err,

    // SelfTyParam
    // SelfTyAlias,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Namespace {
    TypeNS,
    ValueNS,
}

impl Namespace {
    pub fn descr(self) -> &'static str {
        match self {
            Self::TypeNS => "type",
            Self::ValueNS => "value",
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PerNS<T> {
    pub value_ns: T,
    pub type_ns: T,
}

trait PerNSExt<T> {
    fn map<U, F: FnMut(T) -> U>(self, f: F) -> PerNS<U>;

    fn into_iter(self) -> IntoIter<T, 2>;

    fn iter(&self) -> IntoIter<&T, 2>;
}

impl<T> PerNSExt<T> for PerNS<T> {
    fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> PerNS<U> {
        PerNS { value_ns: f(self.value_ns), type_ns: f(self.type_ns) }
    }

    fn into_iter(self) -> IntoIter<T, 2> {
        [self.value_ns, self.type_ns].into_iter()
    }

    fn iter(&self) -> IntoIter<&T, 2> {
        [&self.value_ns, &self.type_ns].into_iter()
    }
}