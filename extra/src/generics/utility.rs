use std::fmt;

use represent::{AnalyzeWith, MakeType, MakeWith, Maker, TypeAnalyzer, TypeSize};

use super::condition::Condition;
//use crate::{Encrypt, MakeError, Sniffer};

#[derive(Debug)]
pub struct Dbg<T>(pub T);

impl<T: Into<u32>> std::convert::From<Dbg<T>> for u32 {
    fn from(dbg: Dbg<T>) -> Self {
        dbg.0.into()
    }
}

impl<D, T: std::fmt::Debug + Condition<D>> Condition<D> for Dbg<T> {
    const FIXED: bool = T::FIXED;

    fn check(data: &D) -> bool {
        let res = T::check(data);
        println!(
            "Result of {:?}::check is {:?}",
            std::any::type_name::<T>(),
            res
        );
        res
    }
}

impl<A: TypeAnalyzer, T: AnalyzeWith<A>> AnalyzeWith<A> for Dbg<T> {
    const CONST_SIZE: TypeSize = T::CONST_SIZE;

    fn fixed_size(analyzer: &A) -> usize {
        T::fixed_size(analyzer)
    }
}

impl<M: Maker + TypeAnalyzer + MakeType<T>, T: fmt::Debug> MakeWith<M> for Dbg<T> {
    fn make_with(maker: &mut M) -> Result<Dbg<T>, M::Error> {
        let inner = maker.make_type()?;
        println!(
            "Result of {:?}::make is {:?}",
            std::any::type_name::<T>(),
            inner
        );
        Ok(Dbg(inner))
    }
}
