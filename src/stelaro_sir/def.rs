use std::array::IntoIter;

use crate::stelaro_ast::NodeId;
use crate::stelaro_common::{DefId, Symbol};
use crate::stelaro_ty::ty::PrimTy;

use super::definitions::DefPathData;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefKind {
    Mod,    // モジュール
    // Struct,
    // Enum,
    // Field,
    Fn, // 関数定義
    // Static, // Static item
    // Const,  // Const item
}


impl DefKind {
    /// DefKindに対する英語の説明を得る。
    pub fn descr(self, def_id: DefId) -> &'static str {
        match self {
            DefKind::Fn => "function",
            DefKind::Mod if def_id.is_stelo_root() && !def_id.is_local() => "stelo",
            DefKind::Mod => "module",
        }
    }

    pub fn descr_ja(self, def_id: DefId) -> &'static str {
        match self {
            DefKind::Fn => "関数",
            DefKind::Mod if def_id.is_stelo_root() && !def_id.is_local() => "ステロ",
            DefKind::Mod => "モジュール",
        }
    }

    pub fn def_path_data(self, name: Option<Symbol>) -> DefPathData {
        match self {
            DefKind::Mod
                    // | DefKind::Struct
                    // | DefKind::Enum
                    // | DefKind::Variant
                        => DefPathData::TypeNs(Some(name.unwrap())),
            DefKind::Fn
                    // | DefKind::Const
                    // | DefKind::ConstParam
                    // | DefKind::Static { .. }
                    // | DefKind::Field
                        => DefPathData::ValueNs(name.unwrap()),
            // DefKind::Ctor => DefPathData::Ctor,
        }
    }
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

impl Res {
    pub fn descr(&self) -> &'static str {
        match *self {
            Res::Def(kind, def_id) => kind.descr(def_id),
            Res::PrimTy(..) => "builtin type",
            Res::Local(..) => "local variable",
            Res::Err => "unresolved item",
        }
    }

    pub fn descr_ja(&self) -> &'static str {
        match *self {
            Res::Def(kind, def_id) => kind.descr_ja(def_id),
            Res::PrimTy(..) => "組み込み型",
            Res::Local(..) => "ローカル変数",
            Res::Err => "未解決のアイテム",
        }
    }
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

    pub fn descr_ja(self) -> &'static str {
        match self {
            Self::TypeNS => "型",
            Self::ValueNS => "値",
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PerNS<T> {
    pub value_ns: T,
    pub type_ns: T,
}

impl<T> PerNS<T> {
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

impl<T> ::std::ops::Index<Namespace> for PerNS<T> {
    type Output = T;

    fn index(&self, ns: Namespace) -> &T {
        match ns {
            Namespace::ValueNS => &self.value_ns,
            Namespace::TypeNS => &self.type_ns,
        }
    }
}

impl<T> ::std::ops::IndexMut<Namespace> for PerNS<T> {
    fn index_mut(&mut self, ns: Namespace) -> &mut T {
        match ns {
            Namespace::ValueNS => &mut self.value_ns,
            Namespace::TypeNS => &mut self.type_ns,
        }
    }
}