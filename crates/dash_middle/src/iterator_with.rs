use std::convert::Infallible;

pub trait InfallibleResult {
    type Ok;
    fn into_ok(self) -> Self::Ok;
}

impl<T> InfallibleResult for Result<T, Infallible> {
    type Ok = T;

    fn into_ok(self) -> T {
        match self {
            Ok(v) => v,
            Err(v) => match v {},
        }
    }
}

impl<A: InfallibleResult, B: InfallibleResult> InfallibleResult for (A, B) {
    type Ok = (A::Ok, B::Ok);

    fn into_ok(self) -> Self::Ok {
        (self.0.into_ok(), self.1.into_ok())
    }
}

pub struct Enumerate<I>(usize, I);

impl<Args, I: IteratorWith<Args>> IteratorWith<Args> for Enumerate<I> {
    // Wrapped in a result because it allows calling `.next_infallible()` if `I` allows it.
    type Item = (Result<usize, Infallible>, I::Item);

    fn next(&mut self, args: Args) -> Option<Self::Item> {
        let cur = self.0;
        self.1.next(args).map(|value| {
            self.0 += 1;
            (Ok(cur), value)
        })
    }
}

/// Just like `Iterator` in the standard library but can take additional arguments
pub trait IteratorWith<Args>: Sized {
    type Item;

    fn next(&mut self, args: Args) -> Option<Self::Item>;
    fn enumerate(self) -> Enumerate<Self> {
        Enumerate(0, self)
    }
}

pub trait InfallibleIteratorWith<Args>: IteratorWith<Args> {
    type Item;

    fn next_infallible(&mut self, args: Args) -> Option<<Self as InfallibleIteratorWith<Args>>::Item>;
}

impl<Args, I> InfallibleIteratorWith<Args> for I
where
    I: IteratorWith<Args>,
    I::Item: InfallibleResult,
{
    type Item = <I::Item as InfallibleResult>::Ok;

    fn next_infallible(&mut self, args: Args) -> Option<<Self as InfallibleIteratorWith<Args>>::Item> {
        self.next(args).map(|res| res.into_ok())
    }
}
