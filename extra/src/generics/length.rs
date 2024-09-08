use std::{convert::TryFrom, marker::PhantomData};

use represent::{
    AnalyzeType, AnalyzeWith, MakeType, MakeWith, Maker, TypeAnalyzer, TypeSize, VisitType,
    VisitWith, Visitor,
};

use super::{slots::Slots, Has};
//use crate::{Encrypt, MakeError, Sniffer};

#[derive(Debug, Default)]
pub struct Length<T>(pub T);
/*
impl<T> std::ops::Deref for Length<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> std::ops::DerefMut for Length<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}*/

#[derive(Debug)]
pub enum LengthError {
    ConstNotEqual {
        const_size: usize,
        from: usize,
    },
    MakeCantFit {
        size: usize,
        ty: &'static str,
    },
    Verify {
        from: usize,
        verified: usize,
    },
    SlotIsEmpty {
        slot: usize,
    },
    SlotNotEqual {
        slot: usize,
        stored: usize,
        from: usize,
    },
    FixedLength {
        expected: usize,
        actual: Option<usize>,
        ty: &'static str,
    },
}

// region: LenConst

#[derive(Debug, Default)]
pub struct LenConst<const LEN: usize>;

impl<D: TypeAnalyzer, const LEN: usize> AnalyzeWith<D> for Length<LenConst<LEN>> {
    const CONST_SIZE: TypeSize = TypeSize::Const(LEN);
}

impl<const LEN: usize> TryFrom<usize> for LenConst<LEN> {
    type Error = LengthError;

    fn try_from(from: usize) -> Result<Self, LengthError> {
        if from == LEN {
            Ok(Self)
        } else {
            Err(LengthError::ConstNotEqual {
                const_size: LEN,
                from,
            })
        }
    }
}

crate::impl_final_const!(
    [const LEN: usize] for LenConst<LEN> = 0
);

crate::impl_make_default!([const LEN: usize] for LenConst<LEN>);

crate::impl_visit_empty!([const LEN: usize] for LenConst<LEN>);

// endregion
// region: LenMake

#[derive(Debug, Default, Clone)]
pub struct LenMake<L, V = ()>(
    pub(crate) usize,
    pub(crate) PhantomData<L>,
    pub(crate) PhantomData<V>,
);
impl<L, V> LenMake<L, V> {
    pub(crate) fn new(len: usize) -> Self {
        LenMake(len, PhantomData, PhantomData)
    }
}

crate::impl_final_const!(
    [LEN: bytemuck::Pod, V] for LenMake<LEN, V> = std::mem::size_of::<LEN>()
);

impl<LEN, D: TypeAnalyzer, V> AnalyzeWith<D> for Length<LenMake<LEN, V>> {
    const CONST_SIZE: represent::TypeSize = represent::TypeSize::Dynamic;

    fn dynamic_size(&self, _analyzer: &D) -> usize {
        self.0.0
    }
}

impl<M, LEN: Into<u32>, V: Verify<usize>> MakeWith<M> for LenMake<LEN, V>
where
    M: Maker + MakeType<LEN>,
{
    fn make_with(maker: &mut M) -> Result<LenMake<LEN, V>, M::Error> {
        let len: LEN = maker.make_type()?;
        let len: u32 = len.into();
        let len = V::verify(len as usize);
        Ok(LenMake::new(len))
    }
}

impl<T, LEN: TryFrom<usize>, V> VisitWith<T> for LenMake<LEN, V>
where
    T: Visitor + VisitType<LEN>,
    LengthError: Into<T::Error>,
{
    fn visit_with(&self, visitor: &mut T) -> Result<(), T::Error> {
        let len = LEN::try_from(self.0)
            .map_err(|_| LengthError::MakeCantFit {
                size: self.0,
                ty: std::any::type_name::<LEN>(),
            })
            .map_err(Into::into)?;

        visitor.visit(&len)?;
        Ok(())
    }
}

impl<LEN: TryFrom<usize>, V: Verify<usize>> TryFrom<usize> for LenMake<LEN, V> {
    type Error = LengthError;

    fn try_from(from: usize) -> Result<Self, LengthError> {
        let verified = V::verify(from);
        if from != verified {
            return Err(LengthError::Verify { from, verified });
        }
        /*let len = LEN::try_from(verified).map_err(|_| {
            LengthError::MakeCantFit{
                type_max: std::mem::size_of::<LEN>(),
                from
            }
        })?;*/
        Ok(Self::new(verified))
    }
}

// endregion
// region: LenSlot

#[derive(Debug, Default)]
pub struct LenSlot<V, const SLOT: usize>(pub(crate) usize, pub(crate) PhantomData<V>);

impl<V, const SLOT: usize> LenSlot<V, SLOT> {
    pub(crate) fn new(len: usize) -> Self {
        LenSlot(len, PhantomData)
    }
}

impl<D: TypeAnalyzer, V, const SLOT: usize> AnalyzeWith<D> for Length<LenSlot<V, SLOT>> {
    const CONST_SIZE: represent::TypeSize = represent::TypeSize::Dynamic;

    fn dynamic_size(&self, _analyzer: &D) -> usize {
        self.0.0
    }
}

crate::impl_final_const!(
    [V, const SLOT: usize] for LenSlot<V, SLOT> = 0
);

impl<M, V: Verify<usize>, const SLOT: usize> MakeWith<M> for LenSlot<V, SLOT>
where
    M: Maker + Has<Slots>,
{
    fn make_with(maker: &mut M) -> Result<LenSlot<V, SLOT>, M::Error> {
        let slots: &Slots = maker.give();
        let len = match slots.load(SLOT) {
            Some(stored) => stored as usize,
            None => panic!("Expect SLOT #{:?} to have value.", SLOT),
        };

        let len = V::verify(len);
        Ok(LenSlot::new(len))
    }
}

impl<T, V, const SLOT: usize> VisitWith<T> for LenSlot<V, SLOT>
where
    T: Visitor + Has<Slots>,
    LengthError: Into<T::Error>,
{
    fn visit_with(&self, visitor: &mut T) -> Result<(), T::Error> {
        let slots: &Slots = visitor.give();
        let len = match slots.load(SLOT) {
            Some(stored) => stored as usize,
            None => panic!("Expect SLOT #{:?} to have value.", SLOT),
        };
        if len != self.0 {
            Err(LengthError::SlotNotEqual {
                slot: SLOT,
                stored: len,
                from: self.0,
            }
            .into())
        } else {
            Ok(())
        }
    }
}

impl<V: Verify<usize>, const SLOT: usize> TryFrom<usize> for LenSlot<V, SLOT> {
    type Error = LengthError;

    fn try_from(from: usize) -> Result<Self, LengthError> {
        let verified = V::verify(from);
        if from != verified {
            return Err(LengthError::Verify { from, verified });
        }
        Ok(Self::new(verified))
    }
}

// endregion
// region: Verify

pub trait Verify<T> {
    //type Error;
    fn verify(val: T) -> T {
        //Result<T, Self::Error> {
        val
    }
}

impl<T> Verify<T> for () {
    //type Error = std::convert::Infallible;
}

// endregion
// region: MaxClamp

#[derive(Debug)]
pub struct MaxClamp<const MAX: usize>;

impl<const MAX: usize> Verify<usize> for MaxClamp<MAX> {
    fn verify(val: usize) -> usize {
        val.min(MAX)
    }
}

// endregion
// region: MaxSkip

#[derive(Debug)]
pub struct MaxSkip<const MAX: usize>;

impl<const MAX: usize> Verify<usize> for MaxSkip<MAX> {
    fn verify(val: usize) -> usize {
        if val > MAX { 0 } else { val }
    }
}

// endregion

// region: LenDelegate

#[derive(Debug, Clone)]
pub struct FixedLen<LEN> {
    actual: Option<usize>,
    fixed: PhantomData<LEN>,
}
impl<LEN> FixedLen<LEN> {
    fn fixed() -> Self {
        Self {
            actual: None,
            fixed: PhantomData,
        }
    }

    fn from_actual_size(actual: usize) -> Self {
        Self {
            actual: Some(actual),
            fixed: PhantomData,
        }
    }
}

impl<LEN> From<usize> for FixedLen<LEN> {
    fn from(value: usize) -> Self {
        Self::from_actual_size(value)
    }
}

impl<LEN, D: TypeAnalyzer + AnalyzeType<LEN>> AnalyzeWith<D> for Length<FixedLen<LEN>> {
    const CONST_SIZE: represent::TypeSize = represent::TypeSize::Fixed;

    fn fixed_size(analyzer: &D) -> usize {
        analyzer.type_fixed_size()
    }

    fn dynamic_size(&self, analyzer: &D) -> usize {
        Self::fixed_size(analyzer)
    }
}

crate::impl_final_const!([LEN] for FixedLen<LEN> = 0);

impl<M, LEN> MakeWith<M> for FixedLen<LEN>
where
    M: Maker,
{
    fn make_with(_maker: &mut M) -> Result<FixedLen<LEN>, M::Error> {
        Ok(Self::fixed())
    }
}

impl<V, LEN> VisitWith<V> for FixedLen<LEN>
where
    V: Visitor + AnalyzeType<LEN>,
    LengthError: Into<V::Error>,
{
    fn visit_with(&self, visitor: &mut V) -> Result<(), V::Error> {
        let expected = visitor.type_fixed_size();
        if self.actual != Some(expected) {
            Err(LengthError::FixedLength {
                expected,
                actual: self.actual,
                ty: std::any::type_name::<LEN>(),
            }
            .into())
        } else {
            Ok(())
        }
    }
}

// endregion
