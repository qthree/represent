use std::{convert::TryInto, default, fmt, marker::PhantomData};

use represent::{
    AnalyzeType, AnalyzeWith, MakeType, MakeWith, TypeAnalyzer, TypeSize, VisitType, VisitWith,
};

use super::length::{Length, LengthError};
use crate::traits::{MakeBlob, VisitBlob};

// region: BigArr
pub struct BigArr<T, LEN>(pub Vec<T>, pub(crate) PhantomData<LEN>);

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>, LEN> serde::Deserialize<'de> for BigArr<T, LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(Vec::<T>::deserialize(deserializer)?, PhantomData))
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize, LEN> serde::Serialize for BigArr<T, LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T: Clone, LEN> Clone for BigArr<T, LEN> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T, LEN> Default for BigArr<T, LEN> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<T, LEN> BigArr<T, LEN> {
    pub fn new_unchecked(vec: Vec<T>) -> Self {
        Self(vec, PhantomData)
    }
}

impl<T: fmt::Debug, LEN> fmt::Debug for BigArr<T, LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_arr("Vec", &self.0, f)
    }
}

impl<D: TypeAnalyzer + AnalyzeType<LEN> + AnalyzeType<Length<LEN>>, T, LEN> AnalyzeWith<D>
    for BigArr<T, LEN>
{
    const CONST_SIZE: TypeSize = size_and_len(
        std::mem::size_of::<T>(),
        <D as AnalyzeType<Length<LEN>>>::TYPE_CONST_SIZE,
        <D as AnalyzeType<LEN>>::TYPE_CONST_SIZE,
    );

    fn fixed_size(analyzer: &D) -> usize {
        let reps = <D as AnalyzeType<Length<LEN>>>::type_fixed_size(analyzer);
        let header = <D as AnalyzeType<LEN>>::TYPE_CONST_SIZE.expect_const();
        header + reps * std::mem::size_of::<T>()
    }

    fn dynamic_size(&self, _analyzer: &D) -> usize {
        let reps = self.0.len();
        let header = <D as AnalyzeType<LEN>>::TYPE_CONST_SIZE.expect_const();
        header + reps * std::mem::size_of::<T>()
    }
}

const fn size_and_len(bytes: usize, reps: TypeSize, header: TypeSize) -> TypeSize {
    let header = header.expect_const();
    match reps {
        TypeSize::Const(size) => TypeSize::Const(header + bytes * size),
        _ => reps,
    }
}

impl<M, T: bytemuck::Pod, LEN> MakeWith<M> for BigArr<T, LEN>
where
    M: MakeType<LEN> + AnalyzeType<Length<LEN>> + MakeBlob,
{
    fn make_with(maker: &mut M) -> Result<BigArr<T, LEN>, M::Error> {
        let len: LEN = maker.make_type()?;
        let len = maker.type_dynamic_size(&Length(len));
        let vec: Vec<T> = maker.make_blob(len)?;
        Ok(BigArr(vec, Default::default()))
    }
}

impl<V, T: bytemuck::Pod, LEN> VisitWith<V> for BigArr<T, LEN>
where
    V: VisitType<LEN> + VisitBlob,
    usize: TryInto<LEN, Error: Into<V::Error>>,
{
    fn visit_with(&self, visitor: &mut V) -> Result<(), V::Error> {
        let len: LEN = self.0.len().try_into().map_err(Into::into)?;
        visitor.visit(&len)?;
        visitor.visit_blob(&self.0)?;
        Ok(())
    }
}

// endregion
// region: BigStr

#[derive(Clone)]
pub struct BigStr<LEN>(pub BigArr<u8, LEN>);

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
enum MaybeStr<'a> {
    Utf8 {
        string: &'a str,
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "impl_serde::is_zero")
        )]
        tail_zeros: usize,
    },
    Bytes {
        bytes: &'a [u8],
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "impl_serde::is_zero")
        )]
        tail_zeros: usize,
    },
}

impl<'a> fmt::Debug for MaybeStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_map();
        match self {
            Self::Utf8 { string, .. } => debug.entry(&"string", string),
            Self::Bytes { bytes, .. } => debug.entry(&"bytes", bytes),
        };
        match self {
            Self::Utf8 { tail_zeros, .. } | Self::Bytes { tail_zeros, .. } if *tail_zeros > 0 => {
                debug.entry(&"tail_zeros", tail_zeros);
            }
            _ => (),
        };
        debug.finish()
    }
}

#[cfg(feature = "serde")]
mod impl_serde {
    use super::*;

    pub(super) fn is_zero(val: &usize) -> bool {
        *val == 0
    }

    impl<'de, LEN> serde::Deserialize<'de> for BigStr<LEN> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let maybe_str = MaybeStr::deserialize(deserializer)?;
            let (bytes, tail_zeros) = match maybe_str {
                MaybeStr::Utf8 { string, tail_zeros } => (string.as_bytes(), tail_zeros),
                MaybeStr::Bytes { bytes, tail_zeros } => (bytes, tail_zeros),
            };
            let mut vec = Vec::with_capacity(bytes.len() + tail_zeros);
            vec.extend_from_slice(bytes);
            if tail_zeros > 0 {
                vec.resize(vec.len() + tail_zeros, 0);
            }
            Ok(Self(BigArr::new_unchecked(vec)))
        }
    }

    impl<LEN> serde::Serialize for BigStr<LEN> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_maybe_str().serialize(serializer)
        }
    }
}

impl<LEN> BigStr<LEN> {
    pub fn new_unchecked(vec: Vec<u8>) -> Self {
        Self(BigArr::new_unchecked(vec))
    }

    fn as_maybe_str(&self) -> MaybeStr {
        let slice = &self.0.0[..];
        let last_non_zero = slice.iter().rposition(|ch| *ch != 0).unwrap_or(0);
        let tail_zeros = slice.len() - last_non_zero;
        match std::str::from_utf8(&slice[..last_non_zero]) {
            Ok(string) => MaybeStr::Utf8 { string, tail_zeros },
            _ => MaybeStr::Bytes {
                bytes: slice,
                tail_zeros,
            },
        }
    }
}

impl<LEN> BigStr<LEN> {
    pub fn new_fixed<A: TypeAnalyzer>(vec: Vec<u8>, analyzer: &A) -> Result<Self, LengthError>
    where
        Length<LEN>: AnalyzeWith<A>,
    {
        represent::AssertConstOrFixed::<A, Length<LEN>>::assert();
        let verified = Length::<LEN>::fixed_size(analyzer);
        let from = vec.len();
        if verified != from {
            Err(LengthError::Verify { from, verified })
        } else {
            Ok(Self(BigArr::new_unchecked(vec)))
        }
    }

    pub fn new_fixed_fill<A: TypeAnalyzer>(
        mut vec: Vec<u8>,
        analyzer: &A,
        fill: u8,
    ) -> Result<Self, LengthError>
    where
        Length<LEN>: AnalyzeWith<A>,
    {
        represent::AssertConstOrFixed::<A, Length<LEN>>::assert();
        let verified = Length::<LEN>::fixed_size(analyzer);
        let from = vec.len();
        if verified < from {
            return Err(LengthError::Verify { from, verified });
        }
        vec.resize(verified, fill);
        Ok(Self(BigArr::new_unchecked(vec)))
    }
}

impl<LEN> fmt::Debug for BigStr<LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self.as_maybe_str();

        <MaybeStr as std::fmt::Debug>::fmt(&str, f)
    }
}

impl<LEN, D: TypeAnalyzer + AnalyzeType<BigArr<u8, LEN>>> AnalyzeWith<D> for BigStr<LEN> {
    const CONST_SIZE: TypeSize = D::TYPE_CONST_SIZE;

    fn fixed_size(analyzer: &D) -> usize {
        D::type_fixed_size(analyzer)
    }

    fn dynamic_size(&self, analyzer: &D) -> usize {
        analyzer.type_dynamic_size(&self.0)
    }
}

impl<M, LEN> MakeWith<M> for BigStr<LEN>
where
    M: MakeType<BigArr<u8, LEN>>,
{
    fn make_with(maker: &mut M) -> Result<BigStr<LEN>, M::Error> {
        Ok(BigStr(maker.make_type()?))
    }
}

impl<V, LEN> VisitWith<V> for BigStr<LEN>
where
    V: VisitType<BigArr<u8, LEN>>,
{
    fn visit_with(&self, visitor: &mut V) -> Result<(), V::Error> {
        visitor.visit(&self.0)
    }
}

// endregiond

fn debug_arr<T: fmt::Debug>(name: &str, arr: &[T], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    const HALF_MAX: usize = 16;
    let len = arr.len();
    let mut debug = f.debug_struct(name);
    debug.field("len", &len);
    if len > HALF_MAX * 2 {
        let (head, _) = arr.split_at(HALF_MAX);
        let (_, tail) = arr.split_at(len - HALF_MAX);
        debug.field("head", &head).field("tail", &tail);
    } else {
        debug.field("body", &arr);
    }
    debug.finish()
}
