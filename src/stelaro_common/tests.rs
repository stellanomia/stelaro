use std::path::PathBuf;
use std::hash::Hasher;

use crate::stelaro_common::*;
use crate::stelaro_common::idx::{Idx, IntoSliceIdx};
use crate::stelaro_common::symbol::{Symbol, Interner};
use crate::stelaro_common::unhash::{UnhashMap, UnhashSet, Unhasher};
use crate::stelaro_common::source_map::{SourceFile, SourceFileId};

#[test]
fn interner_tests() {
    let interner = Interner::new();
    assert_eq!(interner.intern("abc").as_usize(), 1);
    assert_eq!(interner.intern("abc").as_usize(), 1);
    assert_eq!(interner.intern("def").as_usize(), 2);
    assert_eq!(interner.intern("ghi").as_usize(), 3);
    assert_eq!(interner.intern("def").as_usize(), 2);

    assert_eq!("ghi", interner.get(Symbol::new(3)));
    assert_eq!("def", interner.get(Symbol::new(2)));
}

#[test]
fn test_symbol() {
    create_default_session_globals_then(|| {
        let str = "Hello, World!";
        let symbol1 = Symbol::intern(&str[0..5]);
        let symbol2 = Symbol::intern(&str[5..7]);
        let symbol3 = Symbol::intern(&str[7..]);

        assert_eq!("Hello", symbol1.as_str());
        assert_eq!(", ", symbol2.as_str());
        assert_eq!("World!", symbol3.as_str());
    });
}


#[test]
fn test_typed_arena_alloc() {
    let arena: TypedArena<i32> = TypedArena::new();
    let value = arena.alloc(42);
    assert_eq!(*value, 42);
}

#[test]
fn test_typed_arena_with_capacity() {
    let arena: TypedArena<String> = TypedArena::with_capacity(100);

    let value = arena.alloc(String::from("test"));
    assert_eq!(*value, "test");
    assert!(arena.capacity() >= 100)
}

#[test]
fn test_typed_arena_alloc_slice_copy() {
    let arena: TypedArena<i32> = TypedArena::new();
    let slice = &[1, 2, 3, 4, 5];
    let copied = arena.alloc_slice_copy(slice);
    assert_eq!(copied, slice);
    assert_ne!(copied.as_ptr(), slice.as_ptr(), "コピーされたスライスは元のスライスと異なるメモリアドレスを指すはず");
}

#[test]
fn test_typed_arena_default() {
    let arena: TypedArena<u64> = TypedArena::default();
    let value = arena.alloc(123);
    assert_eq!(*value, 123);
}

#[test]
fn test_typed_arena_empty_slice() {
    let arena: TypedArena<i32> = TypedArena::new();
    let empty_slice: &[i32] = &[];
    let copied = arena.alloc_slice_copy(empty_slice);
    assert_eq!(copied.len(), 0);
    assert!(copied.is_empty());
}


#[test]
fn test_stelo_num() {
    let num = SteloNum::new(42);
    assert_eq!(num.as_u32(), 42);
    assert_eq!(usize::from(num), 42);
    assert_eq!(u32::from(num), 42);
}

#[test]
fn test_def_index() {
    let idx = DefIndex::new(10);
    assert_eq!(idx.as_u32(), 10);
    assert_eq!(usize::from(idx), 10);
    assert_eq!(u32::from(idx), 10);

    let idx_from_usize = DefIndex::from(20usize);
    assert_eq!(idx_from_usize.as_u32(), 20);

    let idx_from_u32 = DefIndex::from(30u32);
    assert_eq!(idx_from_u32.as_u32(), 30);
}

#[test]
fn test_def_id_creation_and_properties() {
    let stelo = SteloNum::new(5);
    let index = DefIndex::new(10);
    let def_id = DefId::new(stelo, index);

    assert_eq!(def_id.stelo, stelo);
    assert_eq!(def_id.index, index);
    assert!(!def_id.is_local());
    assert!(!def_id.is_stelo_root());
    assert!(!def_id.is_top_level_module());
    assert_eq!(def_id.as_local(), None);
}

#[test]
fn test_def_id_local_and_root() {
    let index = DefIndex::new(10);
    let local_def_id = DefId::local(index);
    assert_eq!(local_def_id.stelo, LOCAL_STELO);
    assert_eq!(local_def_id.index, index);
    assert!(local_def_id.is_local());
    assert!(!local_def_id.is_stelo_root());
    assert!(!local_def_id.is_top_level_module());
    assert!(local_def_id.as_local().is_some());

    let root_def_id = DefId::local(STELO_ROOT_INDEX);
    assert!(root_def_id.is_local());
    assert!(root_def_id.is_stelo_root());
    assert!(root_def_id.is_top_level_module());
}

#[test]
fn test_local_def_id() {
    let index = DefIndex::new(10);
    let local_id = LocalDefId::new(index);

    assert_eq!(local_id.local_def_index, index);
    assert!(!local_id.is_top_level_module());

    let def_id = local_id.to_def_id();
    assert_eq!(def_id.stelo, LOCAL_STELO);
    assert_eq!(def_id.index, index);

    let converted_back: LocalDefId = def_id.try_into().expect("DefId は LocalDefId に変換できるはず");
    assert_eq!(converted_back.local_def_index, index);

    let root_local_id = STELO_DEF_ID; // STELO_DEF_ID はトップレベルモジュールを表す LocalDefId 定数と想定
    assert!(root_local_id.is_top_level_module());
}

#[test]
fn test_def_path_hash() {
    let stelo_id = StableSteloId(Hash64::new(123));
    let local_hash = Hash64::new(456);
    let path_hash = DefPathHash::new(stelo_id, local_hash);

    assert_eq!(path_hash.stable_stelo_id(), stelo_id);
    assert_eq!(path_hash.local_hash(), local_hash);

    let default_hash = DefPathHash::default();
    assert_eq!(default_hash.0, Fingerprint::ZERO);
}

#[test]
fn test_stable_stelo_id_consistency() {
    create_default_session_globals_then(|| {
        let stelo_name = Symbol::intern("test_stelo");
        let stelo_id1 = StableSteloId::new(stelo_name);
        let stelo_id2 = StableSteloId::new(stelo_name);
        assert_eq!(stelo_id1, stelo_id2, "同じシンボルから生成されたIDは等しいはず");

        let other_name = Symbol::intern("other_stelo");
        let other_id = StableSteloId::new(other_name);
        assert_ne!(stelo_id1, other_id, "異なるシンボルから生成されたIDは異なるはず");
    });
}


#[test]
fn test_fingerprint_combine_operations() {
    let fp1 = Fingerprint::new(1u64, 2u64);
    let fp2 = Fingerprint::new(3u64, 4u64);

    let combined_ordered = fp1.combine(fp2);
    assert_ne!(combined_ordered, fp1);
    assert_ne!(combined_ordered, fp2);

    let combined_commutative1 = fp1.combine_commutative(fp2);
    let combined_commutative2 = fp2.combine_commutative(fp1);
    assert_eq!(combined_commutative1, combined_commutative2, "可換な結合は順序に依存しないはず");
}

#[test]
fn test_fingerprint_byte_conversion_roundtrip() {
    let original_fp = Fingerprint::new(0x1122334455667788u64, 0x99aabbccddeeff00u64);
    let bytes = original_fp.to_le_bytes();
    let restored_fp = Fingerprint::from_le_bytes(bytes);
    assert_eq!(original_fp, restored_fp);
}



#[test]
fn test_hash64_operations() {
    let h1 = Hash64::new(42);
    assert_eq!(h1.as_u64(), 42);

    let h2 = Hash64::new(100);
    let sum = h1.wrapping_add(h2);
    assert_eq!(sum.as_u64(), 142);

    let mut h3 = Hash64::new(10); // 2進数: 1010
    h3 ^= 5u64;                  // 2進数: 0101
    assert_eq!(h3.as_u64(), 15); // 2進数: 1111
}

#[test]
fn test_hash64_edge_cases_wrapping() {
    let h_zero = Hash64::new(0);
    let h_max = Hash64::new(u64::MAX);

    assert_eq!(h_zero.as_u64(), 0);
    assert_eq!(h_max.as_u64(), u64::MAX);

    let sum_overflow = h_max.wrapping_add(Hash64::new(1));
    assert_eq!(sum_overflow.as_u64(), 0, "wrapping_add は0にオーバーフローするはず");
}

#[test]
fn test_hash128_operations() {
    let h1 = Hash128::new(42);
    assert_eq!(h1.as_u128(), 42);

    let h2 = Hash128::new(100);
    let sum = h1.wrapping_add(h2);
    assert_eq!(sum.as_u128(), 142);

    let truncated = h1.truncate(); // Hash64 への切り捨てを想定
    assert_eq!(truncated.as_u64(), 42);

    let large_val = (u64::MAX as u128) + 10;
    let h_large = Hash128::new(large_val);
    let truncated_large = h_large.truncate();
    assert_eq!(truncated_large.as_u64(), (large_val as u64), "切り捨ては下位ビットを取得するはず");
}


#[test]
fn test_idx_trait_for_primitives() {
    let idx_usize: usize = Idx::new(10);
    assert_eq!(idx_usize, 10);
    assert_eq!(idx_usize.index(), 10);

    let idx_u32: u32 = Idx::new(20);
    assert_eq!(idx_u32, 20);
    assert_eq!(idx_u32.index(), 20);

    let mut idx_mut_usize: usize = Idx::new(5);
    idx_mut_usize.increment_by(3);
    assert_eq!(idx_mut_usize, 8);

    let idx_plus_five = idx_usize.plus(5);
    assert_eq!(idx_plus_five, 15);
}

#[test]
fn test_into_slice_idx_trait_for_ranges() {
    use std::ops::{Range, RangeFrom, RangeTo, RangeInclusive, RangeToInclusive};

    let slice_data = [10, 20, 30, 40, 50];

    // Case 1: 単一インデックス (usize)
    let single_idx: usize = 2;
    let resolved_idx_single = <usize as IntoSliceIdx<usize, [i32]>>::into_slice_idx(single_idx);
    // resolved_idx_single を使ったスライスアクセスは、単一の要素を返す
    // そのため、配列でなく比較対象は整数リテラル 30
    assert_eq!(slice_data[resolved_idx_single], 30);

    // Case 2: Range<usize>
    let range_val: Range<usize> = 1..4;
    let resolved_idx_range = <Range<usize> as IntoSliceIdx<usize, [i32]>>::into_slice_idx(range_val);
    assert_eq!(&slice_data[resolved_idx_range], &[20, 30, 40]);

    // Case 3: RangeFrom<usize>
    let range_from_val: RangeFrom<usize> = 3..;
    let resolved_idx_range_from = <RangeFrom<usize> as IntoSliceIdx<usize, [i32]>>::into_slice_idx(range_from_val);
    assert_eq!(&slice_data[resolved_idx_range_from], &[40, 50]);

    // Case 4: RangeTo<usize>
    let range_to_val: RangeTo<usize> = ..3;
    let resolved_idx_range_to = <RangeTo<usize> as IntoSliceIdx<usize, [i32]>>::into_slice_idx(range_to_val);
    assert_eq!(&slice_data[resolved_idx_range_to], &[10, 20, 30]);

    // Case 5: RangeInclusive<usize>
    let range_inclusive_val: RangeInclusive<usize> = 1..=3;
    let resolved_idx_range_inclusive = <RangeInclusive<usize> as IntoSliceIdx<usize, [i32]>>::into_slice_idx(range_inclusive_val);
    assert_eq!(&slice_data[resolved_idx_range_inclusive], &[20, 30, 40]);

    // Case 6: RangeToInclusive<usize>
    let range_to_inclusive_val: RangeToInclusive<usize> = ..=2;
    let resolved_idx_range_to_inclusive = <RangeToInclusive<usize> as IntoSliceIdx<usize, [i32]>>::into_slice_idx(range_to_inclusive_val);
    assert_eq!(&slice_data[resolved_idx_range_to_inclusive], &[10, 20, 30]);
}

#[test]
fn test_index_vec_basic_operations() {
    let mut vec: IndexVec<usize, i32> = IndexVec::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);

    let idx0 = vec.push(10);
    let idx1 = vec.push(20);
    let idx2 = vec.push(30);

    assert_eq!(idx0, 0);
    assert_eq!(idx1, 1);
    assert_eq!(idx2, 2);

    assert_eq!(vec[idx0], 10);
    assert_eq!(vec[idx1], 20);
    assert_eq!(vec[idx2], 30);

    assert_eq!(vec.len(), 3);
    assert!(!vec.is_empty());

    assert_eq!(vec.pop(), Some(30));
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.pop(), Some(20));
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.pop(), Some(10));
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.pop(), None);
}

#[test]
fn test_index_vec_iterators() {
    let mut vec: IndexVec<usize, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let sum: i32 = vec.iter().sum();
    assert_eq!(sum, 60);

    let collected_pairs: Vec<(usize, &i32)> = vec.iter_enumerated().collect();
    assert_eq!(collected_pairs, vec![(0, &10), (1, &20), (2, &30)]);
}


#[test]
fn test_index_vec_creation_methods() {
    let raw_vec_data = vec![1, 2, 3];
    let vec_from_raw: IndexVec<usize, i32> = IndexVec::from_raw(raw_vec_data.clone());
    assert_eq!(vec_from_raw.len(), 3);
    assert_eq!(vec_from_raw[0], 1);

    let vec_with_capacity: IndexVec<usize, i32> = IndexVec::with_capacity(10);
    assert_eq!(vec_with_capacity.len(), 0);
    assert!(vec_with_capacity.raw.capacity() >= 10); // `raw` で内部 Vec にアクセスすると想定

    let vec_from_elem: IndexVec<usize, i32> = IndexVec::from_elem_n(42, 5);
    assert_eq!(vec_from_elem.len(), 5);
    assert!(vec_from_elem.iter().all(|&x| x == 42));

    let vec_from_fn: IndexVec<usize, i32> = IndexVec::from_fn_n(|i| (i as i32) * 10, 5);
    assert_eq!(vec_from_fn.len(), 5);
    for i in 0..5 {
        assert_eq!(vec_from_fn[i], (i as i32) * 10);
    }
}

#[test]
fn test_index_vec_option_element_methods() {
    let mut vec: IndexVec<usize, Option<String>> = IndexVec::new();

    let idx_uninit = 5;
    let val_ref = vec.ensure_contains_elem(idx_uninit, || None);
    assert_eq!(*val_ref, None);
    assert_eq!(vec.len(), idx_uninit + 1);
    assert_eq!(vec[idx_uninit], None);

    let old_val_at_3 = vec.insert(3, "hello".to_string());
    assert_eq!(old_val_at_3, None, "None の位置への挿入は None を返すはず");
    assert_eq!(vec[3], Some("hello".to_string()));

    let old_val_at_3_overwrite = vec.insert(3, "world".to_string());
    assert_eq!(old_val_at_3_overwrite, Some("hello".to_string()), "上書き挿入は古い値を返すはず");
    assert_eq!(vec[3], Some("world".to_string()));

    let val_at_4_ref = vec.get_or_insert_with(4, || "inserted".to_string());
    assert_eq!(*val_at_4_ref, "inserted");
    assert_eq!(vec[4], Some("inserted".to_string()));

    assert!(vec.contains(3), "インデックス3に要素が含まれているはず");
    assert!(vec.contains(4), "インデックス4に要素が含まれているはず");
    assert!(!vec.contains(2), "インデックス2に要素は含まれていないはず (None のまま)");

    let removed_val_at_3 = vec.remove(3);
    assert_eq!(removed_val_at_3, Some("world".to_string()));
    assert!(!vec.contains(3), "削除後、インデックス3に要素は含まれていないはず");
    assert_eq!(vec[3], None, "削除後、インデックス3の要素は None になるはず");
}


#[test]
fn test_index_slice_access_and_properties() {
    let vec_data: IndexVec<usize, i32> = IndexVec::from_raw(vec![10, 20, 30, 40, 50]);
    let slice = vec_data.as_slice();

    assert_eq!(slice.len(), 5);
    assert!(!slice.is_empty());

    assert_eq!(slice[0], 10);
    assert_eq!(slice[4], 50);
    assert_eq!(slice.get(5), None, "範囲外アクセスは None を返すはず");

    assert_eq!(slice.next_index(), 5, "次のインデックスは現在の長さのはず");
    assert_eq!(slice.last_index(), Some(4), "最後の有効なインデックス");

    let empty_vec: IndexVec<usize, i32> = IndexVec::new();
    let empty_slice = empty_vec.as_slice();
    assert!(empty_slice.is_empty());
    assert_eq!(empty_slice.last_index(), None);
}

#[test]
fn test_index_slice_iterators() {
    let vec_data: IndexVec<usize, i32> = IndexVec::from_raw(vec![10, 20, 30, 40, 50]);
    let slice = vec_data.as_slice();

    let sum: i32 = slice.iter().sum();
    assert_eq!(sum, 150);

    let collected_pairs: Vec<(usize, &i32)> = slice.iter_enumerated().collect();
    assert_eq!(collected_pairs, vec![(0, &10), (1, &20), (2, &30), (3, &40), (4, &50)]);

    let collected_indices: Vec<usize> = slice.indices().collect();
    assert_eq!(collected_indices, vec![0, 1, 2, 3, 4]);
}


#[test]
fn test_index_slice_mut_operations() {
    let mut vec_data: IndexVec<usize, i32> = IndexVec::from_raw(vec![10, 20, 30, 40, 50]);
    let slice = vec_data.as_mut_slice();

    slice[2] = 35;
    assert_eq!(slice[2], 35);

    slice.swap(0, 4); // 10 と 50 を入れ替え
    assert_eq!(slice[0], 50);
    assert_eq!(slice[4], 10);
    // 現在の状態: [50, 20, 35, 40, 10]

    if let Some(value_ref) = slice.get_mut(1) { // 20 を取得
        *value_ref = 25;
    }
    assert_eq!(slice[1], 25);
    // 現在の状態: [50, 25, 35, 40, 10]

    for value_ref in slice.iter_mut() {
        *value_ref += 5;
    }
    // 期待される状態: [55, 30, 40, 45, 15]
    assert_eq!(slice[0], 55);
    assert_eq!(slice[1], 30);
    assert_eq!(slice[2], 40);
    assert_eq!(slice[3], 45);
    assert_eq!(slice[4], 15);

    for (i, value_ref) in slice.iter_enumerated_mut() {
        *value_ref += i as i32; // 各要素にインデックスを加算
    }
    // 期待される状態:
    // slice[0] = 55 + 0 = 55
    // slice[1] = 30 + 1 = 31
    // slice[2] = 40 + 2 = 42
    // slice[3] = 45 + 3 = 48
    // slice[4] = 15 + 4 = 19
    assert_eq!(slice[0], 55);
    assert_eq!(slice[1], 31);
    assert_eq!(slice[2], 42);
    assert_eq!(slice[3], 48);
    assert_eq!(slice[4], 19);
}

#[test]
fn test_index_slice_binary_search() {
    let vec_data: IndexVec<usize, i32> = IndexVec::from_raw(vec![10, 20, 30, 40, 50]); // ソート済み
    let slice = vec_data.as_slice();

    assert_eq!(slice.binary_search(&30), Ok(2), "存在する要素の検索");
    assert_eq!(slice.binary_search(&5), Err(0), "全要素より小さい要素の検索");
    assert_eq!(slice.binary_search(&35), Err(3), "中間に存在しない要素の検索");
    assert_eq!(slice.binary_search(&55), Err(5), "全要素より大きい要素の検索");

    let empty_vec: IndexVec<usize, i32> = IndexVec::new();
    assert_eq!(empty_vec.as_slice().binary_search(&10), Err(0));
}

#[test]
fn test_span_properties() {
    let span = Span { start: 10, end: 20 };
    assert_eq!(span.len(), 10);
    assert!(!span.is_empty());

    let empty_span = Span { start: 5, end: 5 };
    assert_eq!(empty_span.len(), 0);
    assert!(empty_span.is_empty());
}

#[test]
fn test_span_operations() {
    let span1 = Span { start: 10, end: 20 };
    let span2 = Span { start: 15, end: 25 };

    let merged = span1.merge(&span2);
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 25);

    let between_span = span1.between(&span2);
    assert_eq!(between_span.start, 20);
    assert_eq!(between_span.end, 15);
}

#[test]
fn test_span_conversions() {
    let range_u32 = 10u32..20u32;
    let span_from_range_u32: Span = range_u32.clone().into();
    assert_eq!(span_from_range_u32.start, 10);
    assert_eq!(span_from_range_u32.end, 20);
    let converted_back_range_u32: std::ops::Range<u32> = span_from_range_u32.into();
    assert_eq!(converted_back_range_u32, range_u32);

    let range_usize = 5usize..15usize;
    let span_from_range_usize: Span = range_usize.clone().into();
    assert_eq!(span_from_range_usize.start, 5);
    assert_eq!(span_from_range_usize.end, 15);
    let converted_back_range_usize: std::ops::Range<usize> = span_from_range_usize.into();
    assert_eq!(converted_back_range_usize, range_usize);

    let tuple_u32 = (30u32, 40u32);
    let span_from_tuple_u32 = Span::from(tuple_u32);
    assert_eq!(span_from_tuple_u32.start, 30);
    assert_eq!(span_from_tuple_u32.end, 40);

    let tuple_usize = (50usize, 60usize);
    let span_from_tuple_usize = Span::from(tuple_usize);
    assert_eq!(span_from_tuple_usize.start, 50);
    assert_eq!(span_from_tuple_usize.end, 60);
}

// ===== Symbol のテスト =====
#[test]
fn test_symbol_interning_and_properties() {
    create_default_session_globals_then(|| {
        let sym_hello1 = Symbol::intern("hello");
        let sym_world = Symbol::intern("world");
        let sym_hello2 = Symbol::intern("hello");

        assert_eq!(sym_hello1.as_str(), "hello");
        assert_eq!(sym_world.as_str(), "world");
        assert_eq!(sym_hello1, sym_hello2, "同じ文字列から作られたシンボルは等しいはず");
        assert_ne!(sym_hello1, sym_world, "異なる文字列から作られたシンボルは異なるはず");

        let underscore_symbol = Symbol::UNDERSCORE;
        assert!(underscore_symbol.is_underscore());
        assert!(!sym_hello1.is_underscore());

        assert_ne!(sym_hello1.as_usize(), underscore_symbol.as_usize(), "ユーザーシンボルのインデックスは UNDERSCORE と異なるはず (もし 'hello' がそれにマップされない限り)");
        assert_ne!(sym_world.as_usize(), underscore_symbol.as_usize());
        assert_ne!(sym_hello1.as_usize(), sym_world.as_usize());
        assert_eq!(sym_hello1.as_usize(), sym_hello2.as_usize());
    });
}

#[test]
fn test_ident_creation_and_properties() {
    create_default_session_globals_then(|| {
        let name_symbol = Symbol::intern("test_ident");
        let span_data = Span { start: 10, end: 20 };
        let ident = Ident::new(name_symbol, span_data);

        assert_eq!(ident.name, name_symbol);
        assert_eq!(ident.span, span_data);
        assert_eq!(ident.name.as_str(), "test_ident");
        assert!(!ident.is_underscore());

        let underscore_ident = Ident::new(Symbol::UNDERSCORE, span_data);
        assert!(underscore_ident.is_underscore());
    });
}


#[test]
fn test_unhasher_simple_write_finish() {
    let mut hasher = Unhasher::default();
    hasher.write_u64(42);
    assert_eq!(hasher.finish(), 42); // 単純な u64 の Unhasher の場合
}


#[test]
fn test_unhash_map_basic_crud() {
    let mut map: UnhashMap<u64, &str> = UnhashMap::default();
    assert!(map.is_empty());

    map.insert(1, "one");
    assert!(!map.is_empty());
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&1), Some(&"one"));
    assert_eq!(map.get(&2), None);

    map.insert(2, "two");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&2), Some(&"two"));

    map.insert(1, "uno"); // 更新
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&"uno"));

    assert_eq!(map.remove(&1), Some("uno")); // 削除
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.remove(&3), None); // 存在しない要素の削除
}

#[test]
fn test_unhash_set_basic_operations() {
    let mut set: UnhashSet<u64> = UnhashSet::default();
    assert!(set.is_empty());

    assert!(set.insert(10)); // 新規挿入
    assert!(!set.is_empty());
    assert_eq!(set.len(), 1);
    assert!(set.contains(&10));
    assert!(!set.contains(&20));

    assert!(!set.insert(10)); // 既存要素の挿入
    assert_eq!(set.len(), 1);

    assert!(set.insert(20));
    assert_eq!(set.len(), 2);
    assert!(set.contains(&20));

    assert!(set.remove(&10)); // 既存要素の削除
    assert_eq!(set.len(), 1);
    assert!(!set.contains(&10));
    assert!(!set.remove(&30)); // 存在しない要素の削除
}


#[test]
fn test_source_file_id_equality() {
    let path1_str = "/path/to/file1.txt";
    let path2_str = "/path/to/file2.txt";
    let path1 = PathBuf::from(path1_str);
    let path2 = PathBuf::from(path2_str);

    let id1 = SourceFileId::from_file_name(&path1);
    let id2 = SourceFileId::from_file_name(&path2);
    let id1_dup = SourceFileId::from_file_name(&PathBuf::from(path1_str));

    assert_eq!(id1, id1_dup, "同じファイルパスから生成されたIDは等しいはず");
    assert_ne!(id1, id2, "異なるファイルパスから生成されたIDは異なるはず");
}

#[test]
fn test_source_file_creation_and_properties() {
    let path_str = "/path/to/test.txt";
    let file_path = PathBuf::from(path_str);
    let content_str = "This is a test file with Unicode 😊.";

    let file = SourceFile::new(file_path.clone(), content_str.to_string());

    assert_eq!(file.name, file_path);
    assert_eq!(*file.src, content_str);

    let expected_id = SourceFileId::from_file_name(&file_path);
    assert_eq!(file.file_id.0.as_u128(), expected_id.0.as_u128(), "File ID は生成されたIDと一致するはず");
}


#[test]
fn test_session_globals_creation_and_access() {
    let result_from_create = create_session_globals_then(None, || {
        let sym = Symbol::intern("symbol_in_create_globals");
        assert_eq!(sym.as_str(), "symbol_in_create_globals");
        100
    });
    assert_eq!(result_from_create, 100);

    create_default_session_globals_then(|| {
        let symbol_in_default = Symbol::intern("symbol_in_default_globals_direct");
        assert_eq!(symbol_in_default.as_str(), "symbol_in_default_globals_direct");

        let result_from_with = with_session_globals(|globals| {
            let sym = globals.symbol_interner.intern("symbol_in_with_globals");
            sym.as_str().to_string()
        });
        assert_eq!(result_from_with, "symbol_in_with_globals");
    });
}