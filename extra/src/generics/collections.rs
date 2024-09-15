use std::{convert::TryFrom, marker::PhantomData};

use represent::{
    AnalyzeWith, MakeType, MakeWith, Maker, TypeAnalyzer, TypeSize, VisitType, VisitWith, Visitor,
};

use super::length::{Length, LengthError};
//use crate::{Encrypt, Farter, MakeError, Sniffer, VisitError};

// region: RepeatExt

#[derive(derivative::Derivative, Clone)]
#[derivative(Debug)]
pub struct RepeatExt<T, LEN>(
    pub Vec<T>,
    #[derivative(Debug = "ignore")] pub(crate) PhantomData<LEN>,
);

#[cfg(feature = "serde")]
mod impl_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl<'de, T: Deserialize<'de>, LEN> Deserialize<'de> for RepeatExt<T, LEN> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self(Vec::<T>::deserialize(deserializer)?, PhantomData))
        }
    }

    impl<T: Serialize, LEN> serde::Serialize for RepeatExt<T, LEN> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0.serialize(serializer)
        }
    }
}

impl<T, LEN> RepeatExt<T, LEN> {
    pub fn new_unchecked(values: Vec<T>) -> Self {
        Self(values, PhantomData)
    }
}

impl<D: TypeAnalyzer, T: AnalyzeWith<D>, LEN: AnalyzeWith<D>> AnalyzeWith<D> for RepeatExt<T, LEN>
where
    Length<LEN>: AnalyzeWith<D>,
{
    const CONST_SIZE: TypeSize =
        repeat_size_and_len(T::CONST_SIZE, Length::<LEN>::CONST_SIZE, LEN::CONST_SIZE);

    fn fixed_size(analyzer: &D) -> usize {
        let reps = LEN::fixed_size(analyzer);
        let bytes = T::fixed_size(analyzer);
        let header = LEN::CONST_SIZE.expect_const();
        header + reps * bytes
    }

    fn dynamic_size(&self, analyzer: &D) -> usize {
        let header = LEN::CONST_SIZE.expect_const();
        let body: usize = self.0.iter().map(|rep| rep.dynamic_size(analyzer)).sum();
        header + body
    }
}

const fn repeat_size_and_len(single: TypeSize, reps: TypeSize, header: TypeSize) -> TypeSize {
    let header = header.expect_const();
    match (single, reps) {
        (TypeSize::Const(bytes), TypeSize::Const(size)) => TypeSize::Const(header + bytes * size),
        (TypeSize::Dynamic, _) => TypeSize::Dynamic,
        (_, TypeSize::Dynamic) => TypeSize::Dynamic,
        _ => TypeSize::Fixed,
    }
}

impl<M, T, LEN> MakeWith<M> for RepeatExt<T, LEN>
where
    Length<LEN>: AnalyzeWith<M>,
    M: MakeType<LEN> + MakeType<T> + Maker + TypeAnalyzer,
{
    fn make_with(maker: &mut M) -> Result<RepeatExt<T, LEN>, M::Error> {
        let len: LEN = maker.make_type()?;
        let len = Length(len).dynamic_size(maker);
        let mut vec = Vec::<T>::with_capacity(len);
        for index in 0..len {
            vec.push(maker.make_keyed(index)?);
        }
        Ok(RepeatExt(vec, Default::default()))
    }
}

impl<V: Visitor, T, LEN> VisitWith<V> for RepeatExt<T, LEN>
where
    LEN: TryFrom<usize, Error = LengthError>,
    V: VisitType<LEN> + VisitType<T> + Visitor,
    <LEN as TryFrom<usize>>::Error: Into<V::Error>,
{
    fn visit_with(&self, visitor: &mut V) -> Result<(), V::Error> {
        let len = LEN::try_from(self.0.len()).map_err(Into::into)?;
        visitor.visit(&len)?;
        for (index, element) in self.0.iter().enumerate() {
            visitor.visit_keyed(index, element)?;
        }
        Ok(())
    }
}

// endregion
