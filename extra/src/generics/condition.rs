use std::marker::PhantomData;

use represent::{
    AnalyzeWith, MakeType, MakeWith, Maker, TypeAnalyzer, TypeSize, VisitType, VisitWith, Visitor,
};

use super::{slots::Slots, Has};

pub trait Condition<D> {
    const FIXED: bool;
    fn check(data: &D) -> bool;
}

// region: Conditional

#[derive(derivative::Derivative)]
#[derivative(Debug, Clone)]
pub struct Conditional<C, T>(
    pub Option<T>,
    #[derivative(Debug = "ignore")] pub(crate) PhantomData<C>,
);

impl<C, T> Default for Conditional<C, T> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<C, T> From<Option<T>> for Conditional<C, T> {
    fn from(value: Option<T>) -> Self {
        Self(value, PhantomData)
    }
}

impl<D: TypeAnalyzer, C: Condition<D>, T: AnalyzeWith<D>> AnalyzeWith<D> for Conditional<C, T> {
    const CONST_SIZE: TypeSize = fixed(C::FIXED);

    fn fixed_size(analyzer: &D) -> usize {
        if C::check(analyzer) {
            T::fixed_size(analyzer)
        } else {
            0
        }
    }

    fn dynamic_size(&self, analyzer: &D) -> usize {
        //Self::fixed_size(analyzer)
        match &self.0 {
            Some(inner) => inner.dynamic_size(analyzer),
            None => 0,
        }
    }
}

const fn fixed(is: bool) -> TypeSize {
    if is {
        TypeSize::Fixed
    } else {
        TypeSize::Dynamic
    }
}

impl<M, C: Condition<M>, T> MakeWith<M> for Conditional<C, T>
where
    M: MakeType<T> + Maker,
{
    fn make_with(maker: &mut M) -> Result<Conditional<C, T>, M::Error> {
        let inner = if C::check(maker) {
            Some(maker.make_type()?)
        } else {
            None
        };
        Ok(Conditional(inner, Default::default()))
    }
}

#[derive(Debug)]
pub enum ConditionalError {
    ConditionalMismatch {
        is_some: bool,
        check: bool,
        type_name: &'static str,
    },
}

impl<V, C: Condition<V>, T> VisitWith<V> for Conditional<C, T>
where
    V: VisitType<T> + Visitor,
    ConditionalError: Into<V::Error>,
{
    fn visit_with(&self, visiter: &mut V) -> Result<(), V::Error> {
        let check = C::check(visiter);
        match (&self.0, check) {
            (Some(inner), true) => visiter.visit(inner),
            (None, false) => Ok(()),
            (inner, check) => Err(ConditionalError::ConditionalMismatch {
                is_some: inner.is_some(),
                check,
                type_name: std::any::type_name::<Conditional<C, T>>(),
            }
            .into()),
        }
    }
}

/*
impl<'b, E: Encrypt, C: Condition<Self>, T> MakeType<Conditional<C, T>> for Sniffer<'b, E>
where
    Self: MakeType<T> + Maker<Error = MakeError>,
{
    fn make_type(&mut self) -> Result<Conditional<C, T>, MakeError> {
        let inner = if C::check(self) {
            Some(self.make_type()?)
        } else {
            None
        };
        Ok(Conditional(inner, Default::default()))
    }
}

impl<'b, E: Encrypt, C: Condition<Self>, T> VisitType<Conditional<C, T>> for Farter<'b, E>
where
    Self: VisitType<T> + Visitor<Error = VisitError>,
{
    fn visit(&mut self, target: &Conditional<C, T>) -> Result<(), VisitError> {
        let check = C::check(self);
        match (&target.0, check) {
            (Some(inner), true) => self.visit(inner),
            (None, false) => Ok(()),
            (inner, check) => Err(VisitError::ConditionalMismatch {
                is_some: inner.is_some(),
                check,
                type_name: std::any::type_name::<Conditional<C, T>>(),
            }),
        }
    }
}
*/

// endregion

#[derive(Debug)]
pub struct Flag<const VAL: u32, const SLOT: usize>;

impl<D: Has<Slots>, const VAL: u32, const SLOT: usize> Condition<D> for Flag<VAL, SLOT> {
    const FIXED: bool = false;

    fn check(data: &D) -> bool {
        let slots = data.give();
        match slots.load(SLOT) {
            Some(stored) => (stored & VAL) != 0,
            None => panic!("Expect SLOT #{:?} to have value.", SLOT),
        }
    }
}

#[derive(Debug)]
pub struct Not<C>(PhantomData<C>);

impl<D, C: Condition<D>> Condition<D> for Not<C> {
    const FIXED: bool = C::FIXED;

    fn check(data: &D) -> bool {
        !C::check(data)
    }
}

#[derive(Debug)]
pub struct Equal<const VAL: u32, const SLOT: usize>;

impl<D: Has<Slots>, const VAL: u32, const SLOT: usize> Condition<D> for Equal<VAL, SLOT> {
    const FIXED: bool = false;

    fn check(data: &D) -> bool {
        let slots = data.give();
        match slots.load(SLOT) {
            Some(stored) => stored == VAL,
            None => panic!("Expect SLOT #{:?} to have value.", SLOT),
        }
    }
}

pub type NotEqual<const VAL: u32, const SLOT: usize> = Not<Equal<VAL, SLOT>>;
