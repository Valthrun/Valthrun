use core::fmt::Display;

/// See: https://learn.microsoft.com/en-us/windows/win32/debug/getexceptioncode
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SEHException {
    AccessViolation = -1073741819i32,
    ArrayBoundsExceeded = -1073741684i32,
    Breakpoint = -2147483645i32,
    DataTypeMisalignment = -2147483646i32,
    FLTDenormalOperand = -1073741683i32,
    FLTDivideByZero = -1073741682i32,
    FLTInexactResult = -1073741681i32,
    FLTInvalidOperation = -1073741680i32,
    FLTOverflow = -1073741679i32,
    FLTStackCheck = -1073741678i32,
    FLTUnderflow = -1073741677i32,
    GuardPage = -2147483647i32,
    IllegalInstruction = -1073741795i32,
    IntDivideByZero = -1073741676i32,
    IntOverflow = -1073741675i32,
    InvalidDisposition = -1073741786i32,
    InvalidHandle = -1073741816i32,
    InPageError = -1073741818i32,
    NonContinuableException = -1073741787i32,
    PrivilegedInstruction = -1073741674i32,
    SingleStep = -2147483644i32,
    StackOverflow = -1073741571i32,
    //UnwindConsolidate = Foundation::STATUS_UNWIND_CONSOLIDATE.0,
}

impl From<i32> for SEHException {
    fn from(err: i32) -> Self {
        unsafe { core::mem::transmute(err) }
    }
}

impl Display for SEHException {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[allow(unreachable_patterns)]
        match self {
            SEHException::AccessViolation => write!(f, "the thread attempts to read from or write to a virtual address for which it does not have access"),
            SEHException::ArrayBoundsExceeded => write!(f, "the thread attempts to access an array element that is out of bounds and the underlying hardware supports bounds checking"),
            SEHException::Breakpoint => write!(f, "a breakpoint was encountered"),
            SEHException::DataTypeMisalignment => write!(f, "the thread attempts to read or write data that is misaligned on hardware that does not provide alignment"),
            SEHException::FLTDenormalOperand => write!(f, "one of the operands in a floating point operation is denormal. A denormal value is one that is too small to represent as a standard floating point value"),
            SEHException::FLTDivideByZero => write!(f, "the thread attempts to divide a floating point value by a floating point divisor of 0 (zero)"),
            SEHException::FLTInexactResult => write!(f, "the result of a floating point operation cannot be represented exactly as a decimal fraction"),
            SEHException::FLTInvalidOperation => write!(f, "this exception represents any floating point exception not included in this list"),
            SEHException::FLTOverflow => write!(f, "the exponent of a floating point operation is greater than the magnitude allowed by the corresponding type"),
            SEHException::FLTStackCheck => write!(f, "the stack has overflowed or underflowed, because of a floating point operation"),
            SEHException::FLTUnderflow => write!(f, "the exponent of a floating point operation is less than the magnitude allowed by the corresponding type"),
            SEHException::GuardPage => write!(f, "the thread accessed memory allocated with the PAGE_GUARD modifier"),
            SEHException::IllegalInstruction => write!(f, "the thread tries to execute an invalid instruction"),
            SEHException::InPageError => write!(f, "the thread tries to access a page that is not present, and the system is unable to load the page"),
            SEHException::IntDivideByZero => write!(f, "the thread attempts to divide an integer value by an integer divisor of 0 (zero)"),
            SEHException::IntOverflow => write!(f, "the result of an integer operation creates a value that is too large to be held by the destination register"),
            SEHException::InvalidDisposition => write!(f, "an exception handler returns an invalid disposition to the exception dispatcher"),
            SEHException::InvalidHandle => write!(f, "the thread used a handle to a kernel object that was invalid"),
            SEHException::NonContinuableException => write!(f, "the thread attempts to continue execution after a non-continuable exception occurs"),
            SEHException::PrivilegedInstruction => write!(f, "the thread attempts to execute an instruction with an operation that is not allowed in the current computer mode"),
            SEHException::SingleStep => write!(f, "a trace trap or other single instruction mechanism signals that one instruction is executed"),
            SEHException::StackOverflow => write!(f, "the thread uses up its stack"),
            value => write!(f, "0x{:x}", *value as u32)
            //SEHException::UnwindConsolidate => write!(f, "a frame consolidation has been executed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure that the size of the SEHException enum is the same as the size of a c_int.
    /// This is important because the exception code is a c_int.
    #[test]
    fn exception_size() {
        assert_eq!(core::mem::size_of::<SEHException>(), core::mem::size_of::<i32>());
    }
}