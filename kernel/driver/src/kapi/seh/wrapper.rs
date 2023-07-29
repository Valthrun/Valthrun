use super::SEHException;

/// The type of a function that can be called by the `seh_stub` function.
type SEHCallback = unsafe extern "system" fn(*mut ());

extern "system" {
    /// Executes a function in an exception-handling block.<br><br>
    ///
    /// # Arguments
    /// * `callback` - Simple function that calls the guarded procedure in the SEH context.
    /// * `closure_ptr` - The procedure to execute in the exception-handling block.
    ///
    /// # Returns
    /// 0 if no exception was thrown, otherwise the exception code.
    fn seh_stub(
        callback: SEHCallback,
        closure_ptr: *mut (),
    ) -> i32;
}

/// Internal function that calls the guarded procedure in the SEH context.<br>
/// This function is called by the `seh_stub` FFI function.<br><br>
///
/// # Arguments
/// * `closure_ptr` - The procedure to execute in the exception-handling block.
unsafe extern "system" fn seh_callback<F>(closure_ptr: *mut ())
    where
        F: FnMut(),
{
    // Convert the raw pointer passed by the C stub function.
    let closure = &mut *(closure_ptr as *mut F);

    // Call the closure passed to try_seh.
    closure();
}

/// Executes a function in a structure-exception-handled block.<br>
/// This is useful for executing code that may throw an exception, and crash
/// the program. (such as a SEGFAULT)<br><br>
///
/// # Arguments
/// * `closure` - The procedure to execute in the exception-handling block.
///
/// # Returns
/// Some if no exception was thrown, None if an exception was thrown.
pub fn try_seh<F>(mut closure: F) -> Result<(), SEHException>
    where
        F: FnMut(),
{
    // Get the raw pointer to the closure.
    let closure_ptr = &mut closure as *mut _ as *mut ();

    // Call the C stub function.
    // This function will call the `seh_callback` function in an SEH block,
    // passing the raw pointer to the closure.
    // The `seh_callback` function will then call the closure.
    match unsafe { seh_stub(seh_callback::<F>, closure_ptr) } {
        0 => Ok(()),
        code => Err(SEHException::from(code)),
    }
}