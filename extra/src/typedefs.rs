use crate::generics::{
    blob::{BigArr, BigStr},
    collections::RepeatExt,
    length::{LenConst, LenMake, LenRest, LenSlot, MaxSkip},
};

pub type RepeatMake<L, T> = RepeatExt<T, LenMake<L>>;
pub type RepeatSlot<T, const SLOT: usize> = RepeatExt<T, LenSlot<(), SLOT>>;

pub type BigArrMake<L, T> = BigArr<T, LenMake<L>>;
pub type BigStaticArr<T, const LEN: usize> = BigArr<T, LenConst<LEN>>;
pub type BigArrSlot<T, const SLOT: usize> = BigArr<T, LenSlot<(), SLOT>>;
pub type BigStrSlot<const SLOT: usize> = BigStr<LenSlot<(), SLOT>>;
pub type BigStrSlotMax<const SLOT: usize, const MAX: usize> = BigStr<LenSlot<MaxSkip<MAX>, SLOT>>;
pub type TailBytes = BigArr<u8, LenRest>;

pub type StaticStr<const LEN: usize> = BigStr<LenConst<LEN>>;
pub type StrMax<L, const MAX: usize> = BigStr<LenMake<L, MaxSkip<MAX>>>;

impl<L, T> RepeatMake<L, T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self::new_unchecked(vec)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<L, T> BigArrMake<L, T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self::new_unchecked(vec)
    }
}

impl TailBytes {
    pub fn new(vec: Vec<u8>) -> Self {
        Self::new_unchecked(vec)
    }
    pub fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

impl<L, const MAX: usize> StrMax<L, MAX> {
    pub fn try_from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() <= MAX {
            Some(Self::new_unchecked(bytes.into()))
        } else {
            None
        }
    }
    pub fn empty() -> Self {
        Self::new_unchecked(vec![])
    }
}

impl<const LEN: usize> StaticStr<LEN> {
    pub fn try_from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() <= LEN {
            let mut vec = bytes.to_vec();
            vec.resize(LEN, 0);
            Some(Self::new_unchecked(vec))
        } else {
            None
        }
    }
    pub fn zeros() -> Self {
        Self::new_unchecked(vec![0; LEN])
    }
}

impl<T: Copy, const LEN: usize> BigStaticArr<T, LEN> {
    pub fn try_from_slice(bytes: &[T]) -> Option<Self> {
        if bytes.len() == LEN {
            let vec = bytes.to_vec();
            Some(Self::new_unchecked(vec))
        } else {
            None
        }
    }
}

impl<L> Default for BigStr<LenMake<L, ()>> {
    fn default() -> Self {
        Self::new_unchecked(Vec::new())
    }
}

