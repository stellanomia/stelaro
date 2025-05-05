//! rustc の `rustc_span/def_id.rs` に基づいて設計されています。

use std::{fmt, hash::Hash};
use super::{fingerprint::Fingerprint, Hash64, Idx, Symbol, StableHasher};

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

/// `DefPathHash` は `DefPath` を固定長ハッシュで表現した構造体です。
/// これは2つの独立した64ビットハッシュから構成されます。
/// 最初のハッシュは、この `DefPathHash` が由来するステロを一意に識別し
/// ([`StableSteloId`] を参照)、次のハッシュはそのステロ内の対応する `DefPath` を
/// 一意に識別します。これらを組み合わせることで、ステログラフ全体で一意な識別子を形成します。
/// 高品質なハッシュ関数を使用すれば、ハッシュの衝突確率は
/// 非常に低いですが、該当するアイテムの名前を変更したり
/// するなどで簡単に回避することができます。
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefPathHash(pub Fingerprint);


impl DefPathHash {
    /// [DefPathHash] の由来元であるステロを示す [StableSteloId] を返します。
    #[inline]
    pub fn stable_stelo_id(&self) -> StableSteloId {
        StableSteloId(self.0.split().0)
    }

    /// [DefPathHash] のうち、ステロにローカルな部分を返します。
    #[inline]
    pub fn local_hash(&self) -> Hash64 {
        self.0.split().1
    }

    /// 指定された [StableSteloId] と `local_hash` を用いて、新しい [DefPathHash] を構築します。
    /// ただし、`local_hash` はそのステロ内で一意でなければなりません。
    pub fn new(stable_stelo_id: StableSteloId, local_hash: Hash64) -> DefPathHash {
        DefPathHash(Fingerprint::new(stable_stelo_id.0, local_hash))
    }
}


/// [`StableSteloId`] は、ステロ名やその他のいくつかのデータを
/// 組み合わせた64ビットのハッシュ値です。[`DefPathHash`] が [`DefId`] に対応するのと同様に、
/// [`SteloNum`] に対応するものです。このIDはコンパイルセッション間で安定しています。
///
/// このIDはハッシュ値であるため、2つの異なるステロが偶然同じ [`StableSteloId`] を持つ可能性が
/// わずかにあります。コンパイラはステロの読み込み時にそのような衝突を検出し、
/// さらなる問題を防ぐためにコンパイルを中断します。
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct StableSteloId(pub Hash64);

impl StableSteloId {
    pub fn new(
        stelo_name: Symbol,
        // 将来的に -Cmetadata のようなビルド固有の識別情報を受け取る場合に備える
        // mut metadata: Vec<String>,
        // 将来的に言語/コンパイラバージョンの違いをハッシュに反映させたい場合に備える
        // cfg_version: &'static str,
    ) -> StableSteloId {
        let mut hasher = StableHasher::new();

        // stelo_name の内部IDはビルドごとに変わる可能性があるため、
        // ハッシュには安定している文字列表現を使う
        stelo_name.as_str().hash(&mut hasher);

        // // -Cmetadata の引数の順序に依存しないようにソート
        // metadata.sort();
        // // 同じ値が複数ある場合は一度だけ反映する
        // metadata.dedup();

        // hasher.write(b"metadata");
        // for s in &metadata {
        //     // `-Cmetadata=ab -Cmetadata=c` と `-Cmetadata=a -Cmetadata=bc` のような
        //     // 紛らわしいケースの区別のため、文字列長も含めてハッシュ化
        //     hasher.write_usize(s.len());
        //     hasher.write(s.as_bytes());
        // }

        // // コンパイラバージョンによってシンボルが衝突しないようにするため、
        // // 明示的にバージョン文字列を含める (rustc と同様)
        // if let Some(val) = std::env::var_os("STELARO_FORCE_STELARO_VERSION") {
        //     hasher.write(val.to_string_lossy().into_owned().as_bytes())
        // } else {
        //     hasher.write(cfg_version.as_bytes())
        // }

        StableSteloId(hasher.finish())
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0.as_u64()
    }
}

impl fmt::LowerHex for StableSteloId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl Default for DefPathHash {
    fn default() -> Self {
        DefPathHash(Fingerprint::ZERO)
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

impl From<usize> for DefIndex {
    fn from(value: usize) -> Self {
        DefIndex(value as u32)
    }
}

impl From<u32> for DefIndex {
    fn from(value: u32) -> Self {
        DefIndex(value)
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