use crate::stelaro_common::TypedArena;
use crate::stelaro_ty::{Ty, TyKind, ty::{FloatTy, IntTy, UintTy}};


pub struct CommonTypes<'tcx> {
    pub unit: Ty<'tcx>,
    pub bool: Ty<'tcx>,
    pub char: Ty<'tcx>,
    pub isize: Ty<'tcx>,
    pub i8: Ty<'tcx>,
    pub i16: Ty<'tcx>,
    pub i32: Ty<'tcx>,
    pub i64: Ty<'tcx>,
    pub i128: Ty<'tcx>,
    pub usize: Ty<'tcx>,
    pub u8: Ty<'tcx>,
    pub u16: Ty<'tcx>,
    pub u32: Ty<'tcx>,
    pub u64: Ty<'tcx>,
    pub u128: Ty<'tcx>,
    pub f32: Ty<'tcx>,
    pub f64: Ty<'tcx>,
    pub str_: Ty<'tcx>,
    pub never: Ty<'tcx>,
}

impl<'tcx> CommonTypes<'tcx> {
    pub fn new(
        interner: &'tcx TypedArena<'tcx, TyKind<'tcx>>,
    ) -> CommonTypes<'tcx> {
        let mk = |ty| Ty(interner.alloc(ty));

        use TyKind::*;
        use IntTy::*;
        use UintTy::*;
        use FloatTy::*;

        CommonTypes {
            unit: mk(Unit),
            bool: mk(Bool),
            char: mk(Char),
            isize: mk(Int(Isize)),
            i8: mk(Int(I8)),
            i16: mk(Int(I16)),
            i32: mk(Int(I32)),
            i64: mk(Int(I64)),
            i128: mk(Int(I128)),
            usize: mk(Uint(Usize)),
            u8: mk(Uint(U8)),
            u16: mk(Uint(U16)),
            u32: mk(Uint(U32)),
            u64: mk(Uint(U64)),
            u128: mk(Uint(U128)),
            f32: mk(Float(F32)),
            f64: mk(Float(F64)),
            str_: mk(Str),
            never: mk(Never),
        }
    }
}
