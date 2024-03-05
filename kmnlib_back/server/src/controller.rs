use std::future::IntoFuture;
use std::marker::PhantomData;

// Original: https://github.com/HalsekiRaika/architectured/blob/e5caa5c7ae801d1aaac6e275b3ea0ef26d6ff26b/server/src/controller.rs
pub trait Intake<I>: 'static + Sync + Send {
    type To;
    fn emit(&self, input: I) -> Self::To;
}

pub trait TryIntake<I>: 'static + Sync + Send {
    type To;
    type Error;
    fn emit(&self, input: I) -> Result<Self::To, Self::Error>;
}

pub trait Exhaust<I>: 'static + Sync + Send {
    type To;
    fn emit(&self, input: I) -> Self::To;
}

pub trait TryExhaust<I>: 'static + Sync + Send {
    type To;
    type Error;
    fn emit(&self, input: I) -> Result<Self::To, Self::Error>;
}

pub struct Controller<T, P, I, D, O> {
    transformer: T,
    presenter: P,
    _i: PhantomData<I>,
    _t: PhantomData<D>,
    _o: PhantomData<O>,
}

impl<T, P, I, D, O> Controller<T, P, I, D, O> {
    pub fn new(transformer: T, presenter: P) -> Self {
        Self {
            transformer,
            presenter,
            _i: PhantomData,
            _t: PhantomData,
            _o: PhantomData,
        }
    }

    fn preset(self) -> P {
        self.presenter
    }
}

impl<T, P, I, D, O> Controller<T, P, I, D, O>
where
    T: Intake<I, To = D>,
{
    pub fn intake(self, input: I) -> Transformed<T, P, I, D, O> {
        Transformed {
            transformed: self.transformer.emit(input),
            controller: self,
            _i: PhantomData,
            _o: PhantomData,
        }
    }
}

impl<T, P, I, D, O> Controller<T, P, I, D, O>
where
    T: TryIntake<I, To = D>,
{
    pub fn try_intake(self, input: I) -> Result<Transformed<T, P, I, D, O>, T::Error> {
        Ok(Transformed {
            transformed: self.transformer.emit(input)?,
            controller: self,
            _i: PhantomData,
            _o: PhantomData,
        })
    }
}

impl<P, O> Controller<(), P, (), (), O>
where
    P: Exhaust<O>,
{
    pub async fn bypass<F, Fut, E>(self, f: F) -> Result<P::To, E>
    where
        F: FnOnce() -> Fut,
        Fut: IntoFuture<Output = Result<O, E>>,
    {
        Ok(self.preset().emit(f().await?))
    }
}

pub struct Transformed<T, P, I, D, O> {
    transformed: D,
    controller: Controller<T, P, I, D, O>,
    _i: PhantomData<I>,
    _o: PhantomData<O>,
}

impl<T, P, I, D, O> Transformed<T, P, I, D, O>
where
    T: Intake<I, To = D>,
    P: Exhaust<O>,
{
    pub async fn handle<F, Fut, E>(self, f: F) -> Result<P::To, E>
    where
        F: FnOnce(D) -> Fut,
        Fut: IntoFuture<Output = Result<O, E>>,
    {
        Ok(self.controller.preset().emit(f(self.transformed).await?))
    }
}

impl<T, P, I, D, O> Transformed<T, P, I, D, O>
where
    T: Intake<I, To = D>,
    P: TryExhaust<O>,
{
    pub async fn try_handle<F, Fut, E>(self, f: F) -> Result<P::To, P::Error>
    where
        F: FnOnce(D) -> Fut,
        Fut: IntoFuture<Output = Result<O, E>>,
        E: Into<P::Error>,
    {
        self.controller
            .preset()
            .emit(f(self.transformed).await.map_err(Into::into)?)
    }
}

impl<T, I, D> Transformed<T, (), I, D, ()> {
    pub async fn bypass<F, Fut, O, E>(self, f: F) -> Result<O, E>
    where
        F: FnOnce(D) -> Fut,
        Fut: IntoFuture<Output = Result<O, E>>,
    {
        f(self.transformed).await
    }
}
