use crate::{stelaro_common::TypedArena, stelaro_session::Session, stelaro_ty::{Ty, TyKind}};


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
        sess: &Session,
    ) -> CommonTypes<'tcx> {
        let mk = |ty| interner.alloc(ty);

        CommonTypes {
            unit: (),
            bool: (),
            char: (),
            isize: (),
            i8: (),
            i16: (),
            i32: (),
            i64: (),
            i128: (),
            usize: (),
            u8: (),
            u16: (),
            u32: (),
            u64: (),
            u128: (),
            f16: (),
            f32: (),
            f64: (),
            f128: (),
            str_: (),
            never: (),
        }
    }
}