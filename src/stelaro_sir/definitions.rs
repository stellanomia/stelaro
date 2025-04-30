use crate::stelaro_common::Symbol;



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
