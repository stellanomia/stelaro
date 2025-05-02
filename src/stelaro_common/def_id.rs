use std::fmt;

use super::Idx;

// NOTE: std, core 実装まで使われない
/// stelo を一意に識別する番号。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SteloNum(u32);

pub const LOCAL_STELO: SteloNum = SteloNum(0);

impl SteloNum {
    #[inline]
    pub fn new(id: u32) -> Self { SteloNum(id) }
    #[inline]
    pub fn as_u32(self) -> u32 { self.0 }
}

impl fmt::Display for SteloNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_u32(), f)
    }
}


impl Idx for SteloNum {
    fn new(idx: usize) -> Self {
        Self(Idx::new(idx))
    }

    fn index(self) -> usize {
        self.into()
    }
}

impl From<SteloNum> for usize {
    fn from(value: SteloNum) -> Self {
        value.0 as usize
    }
}

impl From<SteloNum> for u32 {
    fn from(value: SteloNum) -> Self {
        value.0
    }
}


/// ステロ内の定義を一意に識別するインデックス。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DefIndex(u32);

/// ステロのルートモジュールを表すインデックス。
/// このインデックスを持つ定義は、そのステロのトップレベルモジュールとなる。
pub const STELO_ROOT_INDEX: DefIndex = DefIndex(0);

impl DefIndex {
    #[inline]
    pub fn new(index: u32) -> Self { DefIndex(index) }
    #[inline]
    pub fn as_u32(self) -> u32 { self.0 }
}

impl Idx for DefIndex {
    fn new(idx: usize) -> Self {
        Self(Idx::new(idx))
    }

    fn index(self) -> usize {
        self.into()
    }
}

impl From<DefIndex> for usize {
    fn from(value: DefIndex) -> Self {
        value.0 as usize
    }
}

impl From<DefIndex> for u32 {
    fn from(value: DefIndex) -> Self {
        value.0
    }
}


/// コンパイラ内の全ての定義 (ローカル、外部) を一意に識別するID。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId {
    pub stelo: SteloNum,
    pub index: DefIndex,
}


impl DefId {
    /// 新しい DefId を作成します。
    #[inline]
    pub fn new(stelo: SteloNum, index: DefIndex) -> Self {
        DefId { stelo, index }
    }

    /// ローカルステロの DefId を作成するヘルパー。
    #[inline]
    pub fn local(index: DefIndex) -> Self {
        DefId::new(LOCAL_STELO, index)
    }

    /// この DefId がローカルステロで定義されたものか判定します。
    #[inline]
    pub fn is_local(self) -> bool {
        self.stelo == LOCAL_STELO
    }

    /// この DefId が (いずれかの) ステロのルートモジュールを表すか判定します。
    #[inline]
    pub fn is_stelo_root(self) -> bool {
        self.index == STELO_ROOT_INDEX
    }

    #[inline]
    pub fn is_top_level_module(self) -> bool {
        self.is_local() && self.is_stelo_root()
    }
}

impl std::fmt::Debug for DefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefId({}:{})", self.stelo.0, self.index.0)?;
        if self.is_local() {
            write!(f, " (local)")?;
        }
        if self.is_stelo_root() {
            write!(f, " (root)")?;
        }
        Ok(())
    }
}


/// ローカルステロ内の定義のみを指すことを保証する DefId。
/// LocalDefId は DefId が stelo == LOCAL_STELO のときと等しい。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalDefId {
    pub local_def_index: DefIndex,
}

/// ローカルステロのルートモジュールを表す LocalDefId 定数。
pub const STELO_DEF_ID: LocalDefId = LocalDefId { local_def_index: STELO_ROOT_INDEX };


impl LocalDefId {
    /// 新しい LocalDefId を作成します。
    #[inline]
    pub fn new(index: DefIndex) -> Self {
        LocalDefId { local_def_index: index }
    }

    /// 対応する DefId (stelo = LOCAL_STELO) に変換します。
    #[inline]
    pub fn to_def_id(self) -> DefId {
        DefId::new(LOCAL_STELO, self.local_def_index)
    }

    /// この LocalDefId がステロのルートモジュールを表すか判定します。
    #[inline]
    pub fn is_top_level_module(self) -> bool {
        self.local_def_index == STELO_ROOT_INDEX
    }
}


impl Idx for LocalDefId {
    fn new(idx: usize) -> Self {
        Self { local_def_index: Idx::new(idx) }
    }

    fn index(self) -> usize {
        self.into()
    }
}


impl From<LocalDefId> for usize {
    fn from(value: LocalDefId) -> Self {
        value.local_def_index.0 as usize
    }
}

impl From<LocalDefId> for u32 {
    fn from(value: LocalDefId) -> Self {
        value.local_def_index.0
    }
}

impl TryFrom<DefId> for LocalDefId {
    type Error = DefId; // 変換失敗時は元の DefId を返す

    #[inline]
    fn try_from(def_id: DefId) -> Result<Self, Self::Error> {
        if def_id.is_local() {
            Ok(LocalDefId::new(def_id.index))
        } else {
            Err(def_id)
        }
    }
}


impl From<LocalDefId> for DefId {
    #[inline]
    fn from(local_id: LocalDefId) -> Self {
        local_id.to_def_id()
    }
}


impl std::fmt::Debug for LocalDefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.to_def_id(), f)
    }
}