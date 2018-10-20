use failure::Fail;
use compile::Error;

/// Implements an immutable transformation using the given input.
pub trait Transform<In> {
    type Out;

    fn transform(self, value: In) -> Self::Out;
}

pub trait TryTransform<'n, In> {
    type Out;

    fn try_transform(self, value: In) -> ::std::result::Result<Self::Out, Error<'n>>;
}

pub trait TransformMut<In> {
    type Out;

    fn transform_mut(&mut self, value: In) -> Self::Out;
}

pub trait TryTransformMut<'n, In> {
    type Out;

    fn try_transform_mut(&mut self, value: In) -> ::std::result::Result<Self::Out, Error<'n>>;
}
