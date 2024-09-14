use crate::generics::{
    blob::{BigArr, BigStr},
    collections::RepeatExt,
    length::{LenConst, LenMake, LenRest, LenSlot},
};

pub type RepeatMake<L, T> = RepeatExt<T, LenMake<L>>;
pub type RepeatSlot<T, const SLOT: usize> = RepeatExt<T, LenSlot<(), SLOT>>;

pub type BigArrMake<L, T> = BigArr<T, LenMake<L>>;
pub type BigStaticArr<T, const LEN: usize> = BigArr<T, LenConst<LEN>>;
pub type BigArrSlot<T, const SLOT: usize> = BigArr<T, LenSlot<(), SLOT>>;
pub type TailBytes = BigArr<u8, LenRest>;

pub type StaticStr<const LEN: usize> = BigStr<LenConst<LEN>>;
