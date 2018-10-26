use crate::compile::Error;

/// Implements an immutable transformation using the given input.
pub trait Transform<In> {
    type Out;

    fn transform(self, value: In) -> Self::Out;
}

pub trait TryTransform<In> {
    type Out;

    fn try_transform(self, value: In) -> ::std::result::Result<Self::Out, Error>;
}

pub trait TransformMut<In> {
    type Out;

    fn transform_mut(&mut self, value: In) -> Self::Out;
}

pub trait TryTransformMut<In> {
    type Out;

    fn try_transform_mut(&mut self, value: In) -> ::std::result::Result<Self::Out, Error>;
}
