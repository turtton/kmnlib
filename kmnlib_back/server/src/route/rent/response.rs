use crate::controller::Exhaust;

pub struct Presenter;

impl Exhaust<()> for Presenter {
    type To = ();
    fn emit(&self, input: ()) -> Self::To {
        input
    }
}
