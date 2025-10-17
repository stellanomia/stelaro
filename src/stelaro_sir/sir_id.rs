use std::fmt;

use crate::stelaro_common::{DefId, DefIndex, Idx, LocalDefId, STELO_DEF_ID};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct OwnerId {
    pub def_id: LocalDefId,
}

impl fmt::Debug for OwnerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.def_id, f)
    }
}

impl From<OwnerId> for SirId {
    fn from(owner: OwnerId) -> SirId {
        SirId {
            owner,
            local_id: ItemLocalId::ZERO,
        }
    }
}

impl From<OwnerId> for DefId {
    fn from(value: OwnerId) -> Self {
        value.to_def_id()
    }
}

impl OwnerId {
    #[inline]
    pub fn to_def_id(self) -> DefId {
        self.def_id.to_def_id()
    }
}

impl Idx for OwnerId {
    #[inline]
    fn new(idx: usize) -> Self {
        OwnerId {
            def_id: LocalDefId {
                local_def_index: DefIndex::from_usize(idx),
            },
        }
    }

    #[inline]
    fn index(self) -> usize {
        self.def_id.local_def_index.as_usize()
    }
}

/// 現在のステロ内の SIR のノードを一意に識別します。
/// `owner` とは、そのノードを直接囲んでいる sir::Item のような
/// 最も近いアイテムの `LocalDefId` です。
/// そして `local_id` は、その `owner` の中で一意なIDです。
/// この2階層構造によって、IDの値がより安定します。
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SirId {
    pub owner: OwnerId,
    pub local_id: ItemLocalId,
}

impl fmt::Debug for SirId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SirId({:?}.{:?})", self.owner, self.local_id)
    }
}

impl SirId {
    /// 使用すべきでないローカル ID を示す。
    pub const INVALID: SirId = SirId {
        owner: OwnerId {
            def_id: STELO_DEF_ID,
        },
        local_id: ItemLocalId::INVALID,
    };

    #[inline]
    pub fn expect_owner(self) -> OwnerId {
        assert_eq!(self.local_id.index(), 0);
        self.owner
    }

    #[inline]
    pub fn as_owner(self) -> Option<OwnerId> {
        if self.local_id.index() == 0 {
            Some(self.owner)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_owner(self) -> bool {
        self.local_id.index() == 0
    }

    #[inline]
    pub fn make_owner(owner: LocalDefId) -> Self {
        Self {
            owner: OwnerId { def_id: owner },
            local_id: ItemLocalId::ZERO,
        }
    }
}

impl fmt::Display for SirId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}


stelaro_macros::newtype_index! {
    /// `ItemLocalId` は、`sir::Item` の内部にある要素を、一意に識別します。
    /// ある `ItemLocalId` の数値が、所有者アイテム内におけるノードの位置に
    /// 何らかの形で対応するという保証はありません。
    /// しかし、一つの `owner` 内にある `ItemLocalId` 群は、
    /// 0から始まる隙間のない整数の範囲を占めるという保証があります。
    #[orderable]
    pub struct ItemLocalId {}
}


impl ItemLocalId {
    /// 使われるべきでない単一のローカルIDを表す。
    pub const INVALID: ItemLocalId = ItemLocalId::MAX;
}

/// `STELO_NODE_ID` と `STELO_DEF_ID` に対応する `SirId`。
pub const STELO_SIR_ID: SirId = SirId {
    owner: OwnerId {
        def_id: STELO_DEF_ID,
    },
    local_id: ItemLocalId::ZERO,
};

pub const STELO_OWNER_ID: OwnerId = OwnerId {
    def_id: STELO_DEF_ID,
};
