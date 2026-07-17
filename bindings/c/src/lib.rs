//! The wickra-impact C ABI — the hub every C-capable language links against.
//!
//! The surface is tiny and JSON-shaped, exactly like [`impact_core::Impact`]:
//! construct a handle from a spec JSON, drive it with command JSONs (`set_spec`,
//! `set_spec`, `run`, `version`), read back response JSONs, and free the handle.
//! No impact type crosses the boundary by value — the handle is opaque and the
//! payloads are always UTF-8 JSON strings.
//!
//! Responses use a caller-owned buffer with a length-out protocol (the classic
//! C two-call idiom): call with `out = NULL`, `cap = 0` to learn the length
//! `len`, then allocate `len + 1` and call again. When `len < cap` the response
//! is written immediately. Negative returns are reserved for unusable arguments
//! ([`WICKRA_IMPACT_ERR_NULL`], [`WICKRA_IMPACT_ERR_UTF8`]) and caught panics
//! ([`WICKRA_IMPACT_ERR_PANIC`]); a non-negative return is always the response
//! length. Domain errors come back in-band as `{"ok":false,"error":...}` JSON.

use core::ffi::{c_char, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use impact_core::Impact;

/// A required pointer argument (`handle` or `cmd_json`) was null.
pub const WICKRA_IMPACT_ERR_NULL: i32 = -1;
/// `cmd_json` was not valid UTF-8.
pub const WICKRA_IMPACT_ERR_UTF8: i32 = -2;
/// A panic was caught at the FFI boundary.
pub const WICKRA_IMPACT_ERR_PANIC: i32 = -3;

/// An opaque handle to a backtest instance. Created by [`wickra_impact_new`] and
/// destroyed by [`wickra_impact_free`]; never dereferenced by the caller.
///
/// The handle caches the most recent command's response in `pending` so the
/// two-call length protocol does not run the (potentially expensive) backtest
/// twice. The cache is keyed on the raw command bytes and cleared once the
/// response has been delivered.
pub struct WickraImpact {
    inner: Impact,
    pending: Option<(Vec<u8>, String)>,
}

/// Read a NUL-terminated C string as `&str`, or `None` on null / bad UTF-8.
///
/// # Safety
/// `ptr` must be null or a valid NUL-terminated C string.
unsafe fn opt_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

/// Construct a backtest handle from a spec JSON (`"{}"` defers configuration to a
/// later `set_spec`). Returns null on a null / non-UTF-8 / invalid spec. Free it
/// with [`wickra_impact_free`].
///
/// # Safety
/// `spec_json` must be null or a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn wickra_impact_new(spec_json: *const c_char) -> *mut WickraImpact {
    let result = catch_unwind(|| {
        let spec = unsafe { opt_str(spec_json) }?;
        Impact::new(spec).ok().map(|inner| {
            Box::into_raw(Box::new(WickraImpact {
                inner,
                pending: None,
            }))
        })
    });
    result.ok().flatten().unwrap_or(ptr::null_mut())
}

/// Destroy a backtest handle. Null is a no-op.
///
/// # Safety
/// `handle` must be null or a handle previously returned by
/// [`wickra_impact_new`] and not already freed.
#[no_mangle]
pub unsafe extern "C" fn wickra_impact_free(handle: *mut WickraImpact) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// Apply a command JSON and write the response JSON into the caller's buffer.
///
/// Returns the response length in bytes (excluding the terminating NUL), or a
/// negative error code. When `len < cap`, the response and a trailing NUL have
/// been written to `out`; otherwise `out` is left untouched. Pass `out = NULL`,
/// `cap = 0` to query the length.
///
/// # Safety
/// `handle` must be a valid handle; `cmd_json` a valid NUL-terminated C string;
/// `out` either null or a writable buffer of at least `cap` bytes.
#[no_mangle]
pub unsafe extern "C" fn wickra_impact_command(
    handle: *mut WickraImpact,
    cmd_json: *const c_char,
    out: *mut c_char,
    cap: usize,
) -> i32 {
    if handle.is_null() || cmd_json.is_null() {
        return WICKRA_IMPACT_ERR_NULL;
    }
    let Some(cmd) = (unsafe { opt_str(cmd_json) }) else {
        return WICKRA_IMPACT_ERR_UTF8;
    };
    let store = unsafe { &mut *handle };

    let is_retry = matches!(&store.pending, Some((bytes, _)) if bytes.as_slice() == cmd.as_bytes());
    if !is_retry {
        // `command_json` returns a domain error; only a panic is exceptional.
        let Ok(response) = catch_unwind(AssertUnwindSafe(|| {
            store.inner.command_json(cmd).unwrap_or_else(|e| {
                format!(
                    "{{\"ok\":false,\"error\":{}}}",
                    serde_json::to_string(&e.to_string()).unwrap_or_else(|_| "\"error\"".into())
                )
            })
        })) else {
            return WICKRA_IMPACT_ERR_PANIC;
        };
        store.pending = Some((cmd.as_bytes().to_vec(), response));
    }

    let (len, delivered) = {
        let response = &store.pending.as_ref().expect("pending set above").1;
        let bytes = response.as_bytes();
        let len = bytes.len();
        let delivered = len < cap && !out.is_null();
        if delivered {
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), len);
                *out.add(len) = 0;
            }
        }
        (len, delivered)
    };
    if delivered {
        store.pending = None;
    }
    i32::try_from(len).unwrap_or(i32::MAX)
}

/// The library version as a static NUL-terminated string (do not free).
#[no_mangle]
pub extern "C" fn wickra_impact_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0")
        .as_ptr()
        .cast::<c_char>()
}

#[cfg(test)]
mod tests {
    use super::{
        wickra_impact_command, wickra_impact_free, wickra_impact_new, wickra_impact_version,
        WICKRA_IMPACT_ERR_NULL,
    };
    use core::ffi::{c_char, CStr};
    use std::ffi::CString;
    use std::ptr;

    fn read_buf(buf: &[u8]) -> String {
        CStr::from_bytes_until_nul(buf)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn version_command_round_trip() {
        let empty = CString::new("{}").unwrap();
        let handle = unsafe { wickra_impact_new(empty.as_ptr()) };
        assert!(!handle.is_null());
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        let len = unsafe { wickra_impact_command(handle, cmd.as_ptr(), ptr::null_mut(), 0) };
        assert!(len > 0);
        let mut buf = vec![0u8; usize::try_from(len).unwrap() + 1];
        let len2 = unsafe {
            wickra_impact_command(
                handle,
                cmd.as_ptr(),
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
            )
        };
        assert_eq!(len2, len);
        assert!(read_buf(&buf).contains("\"version\""));
        unsafe { wickra_impact_free(handle) };
    }

    #[test]
    fn run_without_spec_is_in_band_error() {
        let empty = CString::new("{}").unwrap();
        let handle = unsafe { wickra_impact_new(empty.as_ptr()) };
        let cmd = CString::new(r#"{"cmd":"run","data":{"candles":[]}}"#).unwrap();
        let len = unsafe { wickra_impact_command(handle, cmd.as_ptr(), ptr::null_mut(), 0) };
        assert!(len > 0);
        let mut buf = vec![0u8; usize::try_from(len).unwrap() + 1];
        unsafe {
            wickra_impact_command(
                handle,
                cmd.as_ptr(),
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
            );
        }
        assert!(read_buf(&buf).contains("\"ok\":false"));
        unsafe { wickra_impact_free(handle) };
    }

    #[test]
    fn too_small_buffer_leaves_out_untouched() {
        let empty = CString::new("{}").unwrap();
        let handle = unsafe { wickra_impact_new(empty.as_ptr()) };
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        let mut buf = vec![0xAAu8; 3];
        let len = unsafe {
            wickra_impact_command(
                handle,
                cmd.as_ptr(),
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
            )
        };
        assert!(usize::try_from(len).unwrap() >= buf.len());
        assert!(buf.iter().all(|&b| b == 0xAA));
        unsafe { wickra_impact_free(handle) };
    }

    #[test]
    fn invalid_spec_returns_null() {
        let bad = CString::new(r#"{"seed":1}"#).unwrap();
        let handle = unsafe { wickra_impact_new(bad.as_ptr()) };
        assert!(handle.is_null());
    }

    #[test]
    fn null_guards() {
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        let code =
            unsafe { wickra_impact_command(ptr::null_mut(), cmd.as_ptr(), ptr::null_mut(), 0) };
        assert_eq!(code, WICKRA_IMPACT_ERR_NULL);
        let empty = CString::new("{}").unwrap();
        let handle = unsafe { wickra_impact_new(empty.as_ptr()) };
        let code = unsafe { wickra_impact_command(handle, ptr::null(), ptr::null_mut(), 0) };
        assert_eq!(code, WICKRA_IMPACT_ERR_NULL);
        unsafe { wickra_impact_free(handle) };
    }

    #[test]
    fn free_null_is_a_noop() {
        unsafe { wickra_impact_free(ptr::null_mut()) };
    }

    #[test]
    fn version_is_nul_terminated() {
        let p = wickra_impact_version();
        let v = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
        assert_eq!(v, env!("CARGO_PKG_VERSION"));
    }
}
