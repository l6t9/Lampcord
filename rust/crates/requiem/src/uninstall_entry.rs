use std::path::Path;

pub fn same_dir(a: &str, b: &str) -> bool {
    fn norm(s: &str) -> String {
        s.replace('/', "\\").trim_end_matches('\\').to_lowercase()
    }
    norm(a) == norm(b)
}

pub fn sync_version(install: &Path, version: &str) {
    #[cfg(windows)]
    {
        let changed = imp::sync_in(imp::UNINSTALL_ROOT, &install.to_string_lossy(), version);
        if changed > 0 {
            log::info!("Apps & features now shows version {version}");
        }
    }
    #[cfg(not(windows))]
    let _ = (install, version);
}

#[cfg(windows)]
mod imp {
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_ENUMERATE_SUB_KEYS, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
        RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    };

    pub const UNINSTALL_ROOT: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    struct Key(HKEY);

    impl Key {
        fn open(parent: HKEY, path: &str, access: u32) -> Option<Key> {
            let mut key: HKEY = null_mut();

            // SAFETY: `parent` is a valid open key, the path is a NUL-terminated UTF-16 buffer that
            // outlives the call, and `key` is a valid out-pointer.
            (unsafe { RegOpenKeyExW(parent, wide(path).as_ptr(), 0, access, &mut key) }
                == ERROR_SUCCESS)
                .then_some(Key(key))
        }

        fn string(&self, name: &str) -> Option<String> {
            let name = wide(name);
            let (mut kind, mut bytes) = (0u32, 0u32);

            // SAFETY: `self.0` is an open key and `name` is NUL-terminated; passing a null data
            // pointer is the documented way to query the value size.
            if unsafe {
                RegQueryValueExW(
                    self.0,
                    name.as_ptr(),
                    null(),
                    &mut kind,
                    null_mut(),
                    &mut bytes,
                )
            } != ERROR_SUCCESS
                || kind != REG_SZ
            {
                return None;
            }
            let mut buf = vec![0u16; (bytes as usize).div_ceil(2)];

            // SAFETY: `self.0` is an open key, `name` is NUL-terminated, and `buf` holds at least
            // `bytes` bytes as reported by the size query above.
            if unsafe {
                RegQueryValueExW(
                    self.0,
                    name.as_ptr(),
                    null(),
                    &mut kind,
                    buf.as_mut_ptr().cast(),
                    &mut bytes,
                )
            } != ERROR_SUCCESS
            {
                return None;
            }
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..len]))
        }

        fn set_string(&self, name: &str, value: &str) -> bool {
            let value = wide(value);

            // SAFETY: `self.0` is an open key, `name` is NUL-terminated, and the data pointer and
            // byte length describe `value`'s UTF-16 buffer.
            unsafe {
                RegSetValueExW(
                    self.0,
                    wide(name).as_ptr(),
                    0,
                    REG_SZ,
                    value.as_ptr().cast(),
                    (value.len() * 2) as u32,
                ) == ERROR_SUCCESS
            }
        }
    }

    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: `self.0` is an open key owned by this `Key`, and `drop` runs once.
            unsafe { RegCloseKey(self.0) };
        }
    }

    fn subkeys(root: &Key) -> Vec<String> {
        let mut names = Vec::new();
        for index in 0.. {
            let mut buf = [0u16; 256];
            let mut len = buf.len() as u32;

            // SAFETY: `root.0` is an open key, and `buf`/`len` describe a writable buffer of the
            // size passed.
            if unsafe {
                RegEnumKeyExW(
                    root.0,
                    index,
                    buf.as_mut_ptr(),
                    &mut len,
                    null(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                )
            } != ERROR_SUCCESS
            {
                break;
            }
            names.push(String::from_utf16_lossy(&buf[..len as usize]));
        }
        names
    }

    pub fn sync_in(root: &str, install: &str, version: &str) -> usize {
        let Some(root_key) = Key::open(HKEY_CURRENT_USER, root, KEY_ENUMERATE_SUB_KEYS) else {
            return 0;
        };
        let mut changed = 0;
        for name in subkeys(&root_key) {
            let Some(entry) = Key::open(
                HKEY_CURRENT_USER,
                &format!(r"{root}\{name}"),
                KEY_QUERY_VALUE | KEY_SET_VALUE,
            ) else {
                continue;
            };
            let Some(location) = entry.string("InstallLocation") else {
                continue;
            };
            if super::same_dir(&location, install)
                && entry.string("DisplayVersion").as_deref() != Some(version)
                && entry.set_string("DisplayVersion", version)
            {
                changed += 1;
            }
        }
        changed
    }

    #[cfg(test)]
    mod tests {
        use std::process;
        use windows_sys::Win32::System::Registry::{RegCreateKeyW, RegDeleteTreeW};

        use super::*;

        fn create(root: &str, name: &str, values: &[(&str, &str)]) {
            let mut key: HKEY = null_mut();

            assert_eq!(
                // SAFETY: the key path is a NUL-terminated UTF-16 buffer that outlives the call,
                // and `key` is a valid out-pointer.
                unsafe {
                    RegCreateKeyW(
                        HKEY_CURRENT_USER,
                        wide(&format!(r"{root}\{name}")).as_ptr(),
                        &mut key,
                    )
                },
                ERROR_SUCCESS
            );
            let key = Key(key);
            for (n, v) in values {
                assert!(key.set_string(n, v));
            }
        }

        fn version(root: &str, name: &str) -> Option<String> {
            Key::open(
                HKEY_CURRENT_USER,
                &format!(r"{root}\{name}"),
                KEY_QUERY_VALUE,
            )?
            .string("DisplayVersion")
        }

        #[test]
        fn only_the_entry_of_this_install_is_updated_and_only_when_it_differs() {
            let root = format!(r"Software\requiem-uninstall-test-{}", process::id());
            create(
                &root,
                "ours",
                &[
                    ("InstallLocation", r"C:\Apps\Client\"),
                    ("DisplayVersion", "1.0.0"),
                ],
            );
            create(
                &root,
                "elsewhere",
                &[
                    ("InstallLocation", r"C:\Other"),
                    ("DisplayVersion", "1.0.0"),
                ],
            );
            create(&root, "no-location", &[("DisplayVersion", "1.0.0")]);

            assert_eq!(sync_in(&root, r"c:/apps/client", "2.0.0-alpha.1"), 1);
            assert_eq!(version(&root, "ours").as_deref(), Some("2.0.0-alpha.1"));
            assert_eq!(version(&root, "elsewhere").as_deref(), Some("1.0.0"));
            assert_eq!(version(&root, "no-location").as_deref(), Some("1.0.0"));
            assert_eq!(
                sync_in(&root, r"C:\Apps\Client", "2.0.0-alpha.1"),
                0,
                "already current"
            );
            assert_eq!(
                sync_in(&format!(r"{root}\missing"), r"C:\Apps\Client", "3.0.0"),
                0,
                "no such root"
            );

            // SAFETY: the path is a NUL-terminated UTF-16 buffer that outlives the call; the test
            // owns the key it deletes.
            unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, wide(&root).as_ptr()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::same_dir;

    #[test]
    fn folders_compare_without_regard_to_case_slashes_or_a_trailing_separator() {
        assert!(same_dir(r"C:\Users\Me\App", "c:/users/me/app/"));
        assert!(same_dir(r"C:\Users\Me\App\", r"C:\Users\Me\App"));
        assert!(!same_dir(r"C:\Users\Me\App", r"C:\Users\Me\App2"));
        assert!(!same_dir(r"C:\Users\Me\App", ""));
    }
}
