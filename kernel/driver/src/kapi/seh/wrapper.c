#include <excpt.h>

/**
 * The type of a function that can be called by the `seh_stub` function.
 */
typedef void (*seh_callback)(void*);

/**
 * Simple implementation of SEH for Rust.
 * @param callback The procedure to execute in a exception-handled block.
 * @param closure_ptr The closure to pass to the `callback` function.
 * @return 0 if no exception was thrown, otherwise the exception code.
 */
int seh_stub(const seh_callback callback, void* closure_ptr)
{
    __try
    {
        callback(closure_ptr);
        return 0;
    }
    __except(EXCEPTION_EXECUTE_HANDLER)
    {
        return GetExceptionCode();
    }
}