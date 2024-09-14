pub mod blob;
pub mod collections;
pub mod condition;
pub mod length;
pub mod slots;
pub mod utility;

pub trait Has<T> {
    fn give(&self) -> &T;
    fn give_mut(&mut self) -> &mut T;
}

pub trait HasValue<T> {
    type Value;
    fn give_value(&self) -> &Self::Value;
    fn give_value_mut(&mut self) -> &mut Self::Value;
}

impl<T, S: HasValue<T, Value = T>> Has<T> for S {
    fn give(&self) -> &T {
        self.give_value()
    }

    fn give_mut(&mut self) -> &mut T {
        self.give_value_mut()
    }
}

pub trait MaybeHasValue<T> {
    type Value;
    fn maybe_give_value(&self) -> Option<&Self::Value>;
    fn maybe_give_value_mut(&mut self) -> Option<&mut Self::Value>;
    fn maybe_replace_value(&mut self, other: Self::Value) -> Option<Self::Value>;
}

impl<T, S: HasValue<T>> MaybeHasValue<T> for S {
    type Value = <S as HasValue<T>>::Value;

    fn maybe_give_value(&self) -> Option<&Self::Value> {
        Some(self.give_value())
    }

    fn maybe_give_value_mut(&mut self) -> Option<&mut Self::Value> {
        Some(self.give_value_mut())
    }

    fn maybe_replace_value(&mut self, other: Self::Value) -> Option<Self::Value> {
        let dest = self.give_value_mut();
        Some(std::mem::replace(dest, other))
    }
}

pub trait MaybeHas<T> {
    fn maybe_give(&self) -> Option<&T>;
    fn maybe_give_mut(&mut self) -> Option<&mut T>;
    fn maybe_replace(&mut self, other: T) -> Option<T>;
}

impl<T, S: MaybeHasValue<T, Value = T>> MaybeHas<T> for S {
    fn maybe_give(&self) -> Option<&T> {
        self.maybe_give_value()
    }

    fn maybe_give_mut(&mut self) -> Option<&mut T> {
        self.maybe_give_value_mut()
    }

    fn maybe_replace(&mut self, other: T) -> Option<T> {
        self.maybe_replace_value(other)
    }
}
