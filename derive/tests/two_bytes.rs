use represent::{MakeType, MakeWith, Maker, VisitType, VisitWith, Visitor};
use represent_derive::{MakeWith, VisitWith};

#[derive(Debug)]
struct Bytes {
    buf: Vec<u8>,
}

impl Visitor for Bytes {
    type Error = ();
}

impl VisitType<u8> for Bytes {
    fn visit(&mut self, target: &u8) -> Result<(), ()> {
        self.buf.push(*target);
        Ok(())
    }
}

impl Maker for Bytes {
    type Error = ();
}

impl MakeType<u8> for Bytes {
    fn make_type(&mut self) -> Result<u8, ()> {
        if self.buf.is_empty() {
            Err(())
        } else {
            Ok(self.buf.remove(0))
        }
    }
}

impl<T: VisitWith<Self>> VisitType<T> for Bytes {
    fn visit(&mut self, target: &T) -> Result<(), <Self as Visitor>::Error> {
        target.visit_with(self)
    }
}

impl<T: MakeWith<Bytes>> MakeType<T> for Bytes {
    fn make_type(&mut self) -> Result<T, ()> {
        T::make_with(self)
    }
}

#[derive(Debug, VisitWith, MakeWith, PartialEq)]
struct TwoBytes {
    first: u8,
    second: u8,
}

#[derive(Debug, MakeWith, PartialEq, VisitWith)]
#[alt(ty = "u8", err = "()", default = "Err(().into())")]
enum SumType {
    #[alt("1")]
    One(u8, u8),
    #[alt("2")]
    Two { _foo: u8, _bar: u8 },
    #[alt("3")]
    Three,
}

#[test]
fn visit_and_remake() {
    let mut bytes = Bytes { buf: vec![] };
    let two_bytes = TwoBytes {
        first: 1,
        second: 2,
    };

    bytes.visit(&two_bytes).unwrap();
    assert_eq!(&bytes.buf, &[1, 2]);

    let new_two_bytes: TwoBytes = bytes.make().unwrap();
    assert_eq!(&two_bytes, &new_two_bytes);
}

fn st<S: AsRef<[u8]>>(slice: S, res: Result<SumType, ()>) {
    let mut bytes = Bytes {
        buf: slice.as_ref().into(),
    };
    let sum_type: Result<SumType, ()> = bytes.make();
    assert_eq!(&sum_type, &res);
}

#[test]
fn make_sum_type() {
    st([1, 2, 3], Ok(SumType::One(2, 3)));
    st([2, 3, 4], Ok(SumType::Two { _foo: 3, _bar: 4 }));
    st([3], Ok(SumType::Three));

    st([0], Err(()));
    st([4], Err(()));
    st([1], Err(()));
    st([1, 2], Err(()));
    st([2], Err(()));
    st([2, 3], Err(()));
}
