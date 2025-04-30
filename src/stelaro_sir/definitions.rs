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

    // Subportions of items:
    // /// Implicit constructor for a unit or tuple-like struct or enum variant.
    // Ctor,
}
