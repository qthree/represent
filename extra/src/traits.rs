use represent::{Maker, Visitor};

pub trait MakeBlob: Maker {
    fn make_blob<T: bytemuck::Pod>(&mut self, len: usize) -> Result<Vec<T>, Self::Error>;
}

pub trait VisitBlob: Visitor {
    /// length of the blob is already visited
    fn visit_blob<T: bytemuck::Pod>(&mut self, blob: &[T]) -> Result<(), Self::Error>;
}

pub trait BytesLeft {
    fn bytes_left(&self) -> usize;
}

pub trait FixedLength<A> {
    fn fixed_length(analyzer: &A) -> usize;
}
