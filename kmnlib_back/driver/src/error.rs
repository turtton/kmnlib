use kernel::KernelError;

pub(crate) trait ConvertError {
    type Ok;
    fn convert_error(self) -> error_stack::Result<Self::Ok, KernelError>;
}
