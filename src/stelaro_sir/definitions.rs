use std::{
    collections::HashMap,
    fmt::{self, Write},
    hash::Hash,
};

use crate::stelaro_common::{
    DefIndex, DefPathHash, Hash64, IndexVec, LOCAL_STELO, LocalDefId, STELO_ROOT_INDEX,
    StableHasher, StableSteloId, SteloNum, Symbol, sym,
};

/// `DefPathTable` は、`DefIndex` と `DefKey` を相互に対応付けます。
/// 内部的には、`DefPathTable` は `DefKey` の木構造を保持しており、各 `DefKey` にはその親の `DefIndex` が格納されています。
/// ステロごとに1つの `DefPathTable` が存在します。
#[derive(Debug)]
pub struct DefPathTable {
    stable_stelo_id: StableSteloId,
    index_to_key: IndexVec<DefIndex, DefKey>,
    // ここの定義は全て現在のステロに属するため、ローカルハッシュのみを格納します。
    def_path_hashes: IndexVec<DefIndex, Hash64>,
    def_path_hash_to_index: HashMap<Hash64, DefIndex>,
}

impl DefPathTable {
    fn new(stable_stelo_id: StableSteloId) -> DefPathTable {
        DefPathTable {
            stable_stelo_id,
            index_to_key: Default::default(),
            def_path_hashes: Default::default(),
            def_path_hash_to_index: Default::default(),
        }
    }

    fn allocate(&mut self, key: DefKey, def_path_hash: DefPathHash) -> DefIndex {
        // 全ての DefPathHash がローカルステロの StableSteloId を正しく含むことを確認。
        debug_assert_eq!(self.stable_stelo_id, def_path_hash.stable_stelo_id());
        let local_hash = def_path_hash.local_hash();

        let index = {
            let index = DefIndex::from(self.index_to_key.len());
            self.index_to_key.push(key);
            index
        };
        self.def_path_hashes.push(local_hash);
        debug_assert!(self.def_path_hashes.len() == self.index_to_key.len());

        // DefPathHash のハッシュ衝突をチェックする
        // これらの衝突確率は極めて低い。
        if let Some(existing) = self.def_path_hash_to_index.insert(local_hash, index) {
            let def_path1 = DefPath::make(LOCAL_STELO, existing, |idx| self.def_key(idx));
            let def_path2 = DefPath::make(LOCAL_STELO, index, |idx| self.def_key(idx));

            // 衝突した DefPathHash をそのまま使用すると、正当性の問題が発生する可能性があるため、
            // コンパイルを中断する必要があります。
            panic!(
                "{def_path1:?} と {def_path2:?} の間で `DefPathHash` の衝突が見つかりました。\n
                 コンパイルを続行できません。"
            );
        }

        index
    }

    #[inline(always)]
    pub fn def_key(&self, index: DefIndex) -> DefKey {
        self.index_to_key[index]
    }

    #[inline(always)]
    pub fn def_path_hash(&self, index: DefIndex) -> DefPathHash {
        let hash = self.def_path_hashes[index];
        DefPathHash::new(self.stable_stelo_id, hash)
    }

    pub fn enumerated_keys_and_path_hashes(
        &self,
    ) -> impl ExactSizeIterator<Item = (DefIndex, &DefKey, DefPathHash)> {
        self.index_to_key
            .iter_enumerated()
            .map(move |(index, key)| (index, key, self.def_path_hash(index)))
    }
}

/// ノードの定義を保持する定義テーブル。
/// `LocalDefId` や `DefPath` に対応する `DefPathTable` を保持する。
/// また、`LocalDefId` と `SirId` の相互変換を行うマッピングも保持する。
#[derive(Debug)]
pub struct Definitions {
    table: DefPathTable,
    next_disambiguator: HashMap<(LocalDefId, DefPathData), u32>,
}

/// 定義を正確に検索するために使用できる一意な識別子。
/// 定義の親 (存在する場合) のインデックスと、`DisambiguatedDefPathData` を組み合わせて構成される。
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DefKey {
    /// 親に対するパスを表す。
    pub parent: Option<DefIndex>,

    /// このノードの識別子。
    pub disambiguated_data: DisambiguatedDefPathData,
}

impl DefKey {
    pub fn compute_stable_hash(&self, parent: DefPathHash) -> DefPathHash {
        let mut hasher = StableHasher::new();

        parent.hash(&mut hasher);

        let DisambiguatedDefPathData {
            ref data,
            disambiguator,
        } = self.disambiguated_data;

        std::mem::discriminant(data).hash(&mut hasher);
        if let Some(name) = data.get_opt_name() {
            // シンボルインデックスではなく、シンボルの文字列表現を考慮して
            // 安定したハッシュを取得します。
            // 補足: Symbol値そのものにコンパイルセッションを超えた安定性はない。
            name.as_str().hash(&mut hasher);
        }

        disambiguator.hash(&mut hasher);

        // これまでの情報からローカル部分のハッシュ値を計算する。
        let local_hash = hasher.finish();

        // 新しい DefPathHash を構築します。その際、ハッシュのID部分 (StableSteloId)
        // が親から適切にコピーされるようにします。これにより、ステロID部分は
        // この DefPathTable 内の全ての DefPathHash にルートから再帰的に伝播します。
        DefPathHash::new(parent.stable_stelo_id(), local_hash)
    }

    #[inline]
    pub fn get_opt_name(&self) -> Option<Symbol> {
        self.disambiguated_data.data.get_opt_name()
    }
}

/// `DefPathData` と整数の曖昧さ回避子 (disambiguator) の組み合わせ。
/// 通常、この整数は `0` だが、同じ `parent` と `data` を持つ複数の定義がある場合、
/// それらを区別するためにこのフィールドが使用される。
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DisambiguatedDefPathData {
    pub data: DefPathData,
    pub disambiguator: u32,
}

#[derive(Debug, Clone)]
pub struct DefPath {
    /// ステロルートからアイテムに至るパスのデータ。
    pub data: Vec<DisambiguatedDefPathData>,

    /// このパスの起点となるステロを示す。
    pub stelo: SteloNum,
}

impl DisambiguatedDefPathData {
    pub fn fmt_maybe_verbose(&self, writer: &mut impl Write, verbose: bool) -> fmt::Result {
        let name = self.data.get_opt_name().unwrap_or(sym::UNKNOWN);
        if verbose && self.disambiguator != 0 {
            write!(writer, "{}#{}", name, self.disambiguator)
        } else {
            writer.write_str(name.as_str())
        }
    }
}

impl fmt::Display for DisambiguatedDefPathData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_maybe_verbose(f, true)
    }
}

impl DefPath {
    pub fn make<FN>(stelo: SteloNum, start_index: DefIndex, mut get_key: FN) -> DefPath
    where
        FN: FnMut(DefIndex) -> DefKey,
    {
        let mut data = vec![];
        let mut index = Some(start_index);
        loop {
            let p = index.unwrap();
            let key = get_key(p);
            match key.disambiguated_data.data {
                DefPathData::SteloRoot => {
                    assert!(key.parent.is_none());
                    break;
                }
                _ => {
                    data.push(key.disambiguated_data);
                    index = key.parent;
                }
            }
        }
        data.reverse();
        DefPath { data, stelo }
    }

    /// ステロ接頭辞なしで、`DefPath`の文字列表現を返します。
    /// このメソッドは、`TyCtxt`が利用できない場合に便利です。
    pub fn to_string_no_stelo_verbose(&self) -> String {
        let mut s = String::with_capacity(self.data.len() * 16);

        for component in &self.data {
            write!(s, "::{component}").unwrap();
        }

        s
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DefPathData {
    // ルート: これらはルートノード専用です。`def_path` 関数内で特別扱いされます。
    /// ステロのルート (marker)。
    SteloRoot,

    // /// Item `use` を表す。
    // Use,
    /// 型名前空間に属するもの。
    TypeNs(Option<Symbol>),
    /// 値名前空間に属するもの。
    ValueNs(Symbol),
    // アイテムの構成要素:
    // /// ユニット型あるいはタプル様の構造体、またはenumバリアントの暗黙的なコンストラクタ。
    // Ctor,
}

impl Definitions {
    pub fn def_path_table(&self) -> &DefPathTable {
        &self.table
    }

    /// 定義の個数を取得します。
    pub fn def_index_count(&self) -> usize {
        self.table.index_to_key.len()
    }

    #[inline]
    pub fn def_key(&self, id: LocalDefId) -> DefKey {
        self.table.def_key(id.local_def_index)
    }

    #[inline(always)]
    pub fn def_path_hash(&self, id: LocalDefId) -> DefPathHash {
        self.table.def_path_hash(id.local_def_index)
    }

    /// ステロルートから `index` で識別される定義へのパスを返します。
    /// ルートノードはこのパスに含まれません。(i.e. ステロルート自身に対しては空のベクタになります)
    /// 将来的に、インライン化されたアイテムの場合、これは外部ステロにおけるアイテムのパスになります。
    /// (ただし、パスは外部ステロへのパスから始まります)
    pub fn def_path(&self, id: LocalDefId) -> DefPath {
        DefPath::make(LOCAL_STELO, id.local_def_index, |index| {
            self.def_key(LocalDefId {
                local_def_index: index,
            })
        })
    }

    /// (親を持たない) ルート定義と、その他のいくつかの予約済み定義を持つ新しい `Definitions` を作成します。
    pub fn new(stable_stelo_id: StableSteloId) -> Definitions {
        let key = DefKey {
            parent: None,
            disambiguated_data: DisambiguatedDefPathData {
                data: DefPathData::SteloRoot,
                disambiguator: 0,
            },
        };

        let parent_hash = DefPathHash::new(stable_stelo_id, Hash64::ZERO);
        let def_path_hash = key.compute_stable_hash(parent_hash);

        // ルートとなる定義を作成する
        let mut table = DefPathTable::new(stable_stelo_id);
        let root = LocalDefId {
            local_def_index: table.allocate(key, def_path_hash),
        };
        assert_eq!(root.local_def_index, STELO_ROOT_INDEX);

        Definitions {
            table,
            next_disambiguator: Default::default(),
        }
    }

    pub fn create_def(&mut self, parent: LocalDefId, data: DefPathData) -> LocalDefId {
        // ルートノードは create_def() によって作られるべきではない
        assert!(data != DefPathData::SteloRoot);

        let disambiguator = {
            let next_disamb = self.next_disambiguator.entry((parent, data)).or_insert(0);
            let disambiguator = *next_disamb;
            *next_disamb = next_disamb.checked_add(1).expect("disambiguator overflow");
            disambiguator
        };

        let key = DefKey {
            parent: Some(parent.local_def_index),
            disambiguated_data: DisambiguatedDefPathData {
                data,
                disambiguator,
            },
        };

        let parent_hash = self.table.def_path_hash(parent.local_def_index);
        let def_path_hash = key.compute_stable_hash(parent_hash);

        LocalDefId {
            local_def_index: self.table.allocate(key, def_path_hash),
        }
    }
}

impl DefPathData {
    pub fn get_opt_name(&self) -> Option<Symbol> {
        use self::DefPathData::*;
        match *self {
            TypeNs(name) => name,

            ValueNs(name) => Some(name),

            Self::SteloRoot => None,
        }
    }
}
