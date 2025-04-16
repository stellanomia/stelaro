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