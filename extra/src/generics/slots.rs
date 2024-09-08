use std::convert::{TryFrom, TryInto};

use represent::{
    AnalyzeWith, MakeType, MakeWith, Maker, TypeAnalyzer, TypeSize, VisitType, VisitWith, Visitor,
};

use super::Has;

#[derive(Debug, Default)]
pub struct Slots([Option<u32>; 16]);

impl Slots {
    pub fn store(&mut self, slot: usize, val: u32) -> Result<(), u32> {
        match &mut self.0[slot] {
            Some(old) => {
                if *old != val {
                    return Err(*old);
                }
            }
            none => *none = Some(val),
        };
        Ok(())
    }

    pub fn load(&self, slot: usize) -> Option<u32> {
        self.0[slot]
    }
}

#[derive(Debug, Clone)]
pub struct Store<T, const SLOT: usize> {
    pub inner: T,
}

impl<D: TypeAnalyzer, T: AnalyzeWith<D>, const SLOT: usize> AnalyzeWith<D> for Store<T, SLOT> {
    const CONST_SIZE: TypeSize = T::CONST_SIZE;

    fn fixed_size(analyzer: &D) -> usize {
        T::fixed_size(analyzer)
    }
}

impl<M, T, const SLOT: usize> MakeWith<M> for Store<T, SLOT>
where
    M: MakeType<T> + Maker + Has<Slots>,
    T: Into<u32> + Clone,
{
    fn make_with(maker: &mut M) -> Result<Store<T, SLOT>, M::Error> {
        let inner = maker.make_type()?;
        let val = inner.clone().into();
        let slots: &mut Slots = maker.give_mut();
        match slots.store(SLOT, val) {
            Ok(_) => Ok(Store { inner }),
            Err(old) => panic!(
                "Expect SLOT #{:?} to be empty or to have value {:?}, got old value {:?} instead.",
                SLOT, val, old
            ),
        }
    }
}

impl<V, T, const SLOT: usize> VisitWith<V> for Store<T, SLOT>
where
    V: VisitType<T> + Visitor + Has<Slots>,
    T: Into<u32> + Clone,
{
    fn visit_with(&self, visitor: &mut V) -> Result<(), V::Error> {
        let slots: &mut Slots = visitor.give_mut();
        let val = self.inner.clone().into();
        match slots.store(SLOT, val) {
            Ok(_) => visitor.visit(&self.inner),
            Err(old) => panic!(
                "Expect SLOT #{:?} to be empty or to have value {:?}, got old value {:?} instead.",
                SLOT, val, old
            ),
        }
    }
}
/*
impl<'b, E: Encrypt, T, const SLOT: usize> MakeType<Store<T, SLOT>> for Sniffer<'b, E>
where
    Self: MakeType<T> + Maker<Error = MakeError>,
    T: Into<u32> + Clone,
{
    fn make_type(&mut self) -> Result<Store<T, SLOT>, MakeError> {
        let inner = self.make_type()?;
        let val = inner.clone().into();
        let slots: &mut Slots = self.give_mut();
        match slots.store(SLOT, val) {
            Ok(_) => Ok(Store { inner }),
            Err(old) => panic!(
                "Expect SLOT #{:?} to be empty or to have value {:?}, got old value {:?} instead.",
                SLOT, val, old
            ),
        }
    }
}

impl<'b, E: Encrypt, T, const SLOT: usize> VisitType<Store<T, SLOT>> for Farter<'b, E>
where
    Self: VisitType<T> + Visitor<Error = VisitError>,
    T: Into<u32> + Clone,
{
    fn visit(&mut self, target: &Store<T, SLOT>) -> Result<(), VisitError> {
        let slots: &mut Slots = self.give_mut();
        let val = target.inner.clone().into();
        match slots.store(SLOT, val) {
            Ok(_) => self.visit(&target.inner),
            Err(old) => panic!(
                "Expect SLOT #{:?} to be empty or to have value {:?}, got old value {:?} instead.",
                SLOT, val, old
            ),
        }
    }
}
*/

#[derive(Debug)]
pub enum SlotLoadError<E> {
    EmptySlot,
    TryFrom(E),
}

#[derive(Debug, Clone)]
pub struct Load<T, const SLOT: usize>(pub T);

impl<M, T, const SLOT: usize> MakeWith<M> for Load<T, SLOT>
where
    M: Maker + Has<Slots>,
    T: TryFrom<u32>,
    SlotLoadError<<T as TryFrom<u32>>::Error>: Into<M::Error>,
{
    fn make_with(maker: &mut M) -> Result<Load<T, SLOT>, M::Error> {
        let slots: &Slots = maker.give();
        match slots.load(SLOT) {
            Some(value) => value.try_into().map(Load).map_err(SlotLoadError::TryFrom),
            None => Err(SlotLoadError::EmptySlot),
        }
        .map_err(Into::into)
    }
}

impl<V, T, const SLOT: usize> VisitWith<V> for Load<T, SLOT>
where
    V: VisitType<T> + Visitor + Has<Slots>,
    T: Into<u32> + Clone,
{
    fn visit_with(&self, _visitor: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
