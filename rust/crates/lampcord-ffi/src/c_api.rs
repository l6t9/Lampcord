use std::ffi::{CStr, CString, c_char};

use crate::{ClientConfig, LampcordClient};

unsafe fn read_str(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

fn into_c_string(value: String) -> *mut c_char {
    CString::new(value)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_new(
    api_base: *const c_char,
    storage_path: *const c_char,
) -> *mut LampcordClient {
    let mut config = ClientConfig::default();
    if let Some(api_base) = unsafe { read_str(api_base) } {
        config.api_base = api_base;
    }
    config.storage_path = unsafe { read_str(storage_path) };
    Box::into_raw(Box::new(LampcordClient::start(config)))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_free(client: *mut LampcordClient) {
    if !client.is_null() {
        drop(unsafe { Box::from_raw(client) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_snapshot_json(
    client: *const LampcordClient,
) -> *mut c_char {
    let Some(client) = (unsafe { client.as_ref() }) else {
        return std::ptr::null_mut();
    };
    match serde_json::to_string(&client.snapshot()) {
        Ok(json) => into_c_string(json),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_connect_token(
    client: *const LampcordClient,
    token: *const c_char,
) -> bool {
    let (Some(client), Some(token)) = (unsafe { client.as_ref() }, unsafe { read_str(token) })
    else {
        return false;
    };
    client.connect_token(token).is_ok()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_connect_stored_token(
    client: *const LampcordClient,
) -> bool {
    unsafe { client.as_ref() }.is_some_and(|client| client.connect_stored_token().is_ok())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_select_guild(
    client: *const LampcordClient,
    guild_id: u64,
) -> bool {
    unsafe { client.as_ref() }.is_some_and(|client| {
        client
            .select_guild((guild_id != 0).then_some(guild_id))
            .is_ok()
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lampcord_engine_select_channel(
    client: *const LampcordClient,
    channel_id: u64,
) -> bool {
    unsafe { client.as_ref() }.is_some_and(|client| {
        client
            .select_channel((channel_id != 0).then_some(channel_id))
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn c_bridge_creates_an_engine_and_reads_a_snapshot() {
        let dir = std::env::temp_dir().join(format!("lampcord-c-api-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let storage =
            CString::new(dir.join("accounts.json").to_string_lossy().into_owned()).expect("path");
        let api = CString::new("https://discord.com/api/v9").expect("api");
        unsafe {
            let client = lampcord_engine_new(api.as_ptr(), storage.as_ptr());
            assert!(!client.is_null());
            let json = lampcord_engine_snapshot_json(client);
            assert!(!json.is_null());
            let text = CStr::from_ptr(json).to_str().expect("utf8").to_owned();
            assert!(text.contains("\"schema_version\""));
            lampcord_string_free(json);
            assert!(lampcord_engine_select_guild(client, 0));
            lampcord_engine_free(client);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
