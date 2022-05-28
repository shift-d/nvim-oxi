use crate::lua::{self, ffi, macros::*};
use crate::Result;

/// Binding to the global Lua `print` function. It uses the same syntax as
/// Rust's `format!` macro and redirects its output to the Neovim message area.
///
/// # Examples
///
/// ```rust
/// nvim_oxi::print!("Hello {planet}!", planet = "Mars");
/// ```
#[macro_export]
macro_rules! nprint {
    ($($arg:tt)*) => {{
        let _ = crate::print(::std::fmt::format(format_args!($($arg)*)));
    }}
}

pub use nprint as print;

/// Prints a message to the Neovim message area. Fails if the provided string
/// contains a null byte.
#[doc(hidden)]
pub fn print(text: impl Into<String>) -> Result<()> {
    let text = std::ffi::CString::new(text.into())?;

    lua::with_state(move |lstate| unsafe {
        ffi::lua_getglobal(lstate, cstr!("print"));
        ffi::lua_pushstring(lstate, text.as_ptr());
        ffi::lua_call(lstate, 1, 0);
    });

    Ok(())
}

/// Binding to `vim.schedule`.
///
/// Schedules a callback to be invoked soon by the main event-loop. Useful to
/// avoid textlock or other temporary restrictions.
pub fn schedule<F>(fun: F)
where
    F: FnOnce(()) -> crate::Result<()> + 'static,
{
    // https://github.com/neovim/neovim/blob/master/src/nvim/lua/executor.c#L316
    //
    // Unfortunately the `nlua_schedule` C function is not exported (it's
    // static), so we need to call the Lua function instead.
    lua::with_state(move |lstate| unsafe {
        // Put `vim.schedule` on the stack.
        ffi::lua_getglobal(lstate, cstr!("vim"));
        ffi::lua_getfield(lstate, -1, cstr!("schedule"));

        // Store the function in the registry and put a reference to it on the
        // stack.
        let luaref = lua::once_to_luaref(fun);
        ffi::lua_rawgeti(lstate, ffi::LUA_REGISTRYINDEX, luaref);

        ffi::lua_call(lstate, 1, 0);

        // Pop `vim` off the stack and remove the reference from the registry.
        ffi::lua_pop(lstate, 1);
        ffi::luaL_unref(lstate, ffi::LUA_REGISTRYINDEX, luaref);
    });
}