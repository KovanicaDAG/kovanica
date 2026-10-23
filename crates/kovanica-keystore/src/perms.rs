//! Owner-only permissions for keystore and seed files.
//!
//! Unix uses mode `0600`. Windows has no mode bits, so a private file gets a
//! protected DACL that allows only the owner (`D:P(A;;FA;;;OW)`). A seed file
//! is rejected when its DACL grants Everyone, Builtin Users, or Authenticated
//! Users. Other platforms have no equivalent in the standard library; the mode
//! check there is a documented no-op after the regular-file check.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

#[cfg(windows)]
const DACL_SECURITY_INFORMATION: u32 = 0x4;
#[cfg(windows)]
const SDDL_REVISION_1: u32 = 1;

/// SDDL for a protected, owner-only allow ACE (full access, no inheritance).
#[cfg(windows)]
const OWNER_ONLY_SDDL: &str = "D:P(A;;FA;;;OW)";

/// Trustees that make a file readable beyond its owner and administrators.
///
/// `WD` / `S-1-1-0` is Everyone, `BU` / `S-1-5-32-545` is Builtin Users, and
/// `AU` / `S-1-5-11` is Authenticated Users. SYSTEM (`SY`) and Administrators
/// (`BA`) are left in place: on Unix, root can still read mode `0600`.
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
const BROAD_TRUSTEES: &[&str] = &["WD", "BU", "AU", "S-1-1-0", "S-1-5-11", "S-1-5-32-545"];

/// Whether `sddl` is a DACL that does not grant broad read access.
///
/// An empty or missing DACL is not owner-only. `NO_ACCESS_CONTROL` is a NULL
/// DACL, which Windows treats as access for everyone.
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) fn dacl_is_owner_only(sddl: &str) -> bool {
    let upper = sddl.to_ascii_uppercase();
    if !upper.contains("D:") || upper.contains("NO_ACCESS_CONTROL") {
        return false;
    }
    let dacl = upper.split("S:").next().unwrap_or(&upper);
    for ace in dacl.split('(').skip(1) {
        let body = ace.split(')').next().unwrap_or("");
        let mut fields = body.split(';');
        let ace_type = fields.next().unwrap_or("");
        if ace_type != "A" && ace_type != "OA" {
            continue;
        }
        let account = fields.nth(4).unwrap_or("");
        if BROAD_TRUSTEES.contains(&account) {
            return false;
        }
    }
    true
}

/// Reject a symlink or a file that other users can read.
pub fn ensure_owner_only(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("reading key file metadata {}", path.display()))?;
    if meta.file_type().is_symlink() {
        bail!(
            "refusing to read key file {} through a symlink",
            path.display()
        );
    }
    if !meta.is_file() {
        bail!("key file {} is not a regular file", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            bail!(
                "key file {} is readable by other users (mode {:o}). Keep it owner-only, for example mode 0600.",
                path.display(),
                meta.permissions().mode() & 0o777
            );
        }
    }
    #[cfg(windows)]
    {
        let sddl = windows_dacl_sddl(path)?;
        if !dacl_is_owner_only(&sddl) {
            bail!(
                "key file {} is readable by other users. Move it to a private folder or remove inherited access for Everyone, Users, and Authenticated Users.",
                path.display()
            );
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        // No portable permission model. The regular-file check above is the
        // whole gate on this platform.
        let _ = meta;
    }
    Ok(())
}

/// Restrict `file` to its owner.
///
/// On Unix this is mode `0600` on the open descriptor. On Windows the open
/// handle's DACL is replaced with [`OWNER_ONLY_SDDL`]. Other platforms are a
/// documented no-op.
pub fn tighten_private_file(file: &fs::File) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(windows)]
    {
        windows_set_owner_only(file)?;
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = file;
    }
    Ok(())
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn windows_set_owner_only(file: &fs::File) -> Result<()> {
    use std::os::windows::io::AsRawHandle;

    let sddl = wide(OWNER_ONLY_SDDL);
    unsafe {
        let mut descriptor = std::ptr::null_mut();
        let ok = ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            std::ptr::null_mut(),
        );
        if ok == 0 {
            return Err(os_error("building an owner-only DACL"));
        }
        // SAFETY: `descriptor` came from the conversion above and is freed
        // below. The handle is the open keystore file this process created.
        let applied = SetKernelObjectSecurity(
            file.as_raw_handle() as *mut std::ffi::c_void,
            DACL_SECURITY_INFORMATION,
            descriptor,
        );
        LocalFree(descriptor);
        if applied == 0 {
            return Err(os_error("restricting the keystore DACL to its owner"));
        }
    }
    Ok(())
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn windows_dacl_sddl(path: &Path) -> Result<String> {
    use std::os::windows::ffi::OsStrExt;

    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let mut needed = 0u32;
        let _ = GetFileSecurityW(
            wide_path.as_ptr(),
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
        if needed == 0 {
            return Err(os_error("reading the key file DACL"));
        }
        let mut buffer = vec![0u8; needed as usize];
        let ok = GetFileSecurityW(
            wide_path.as_ptr(),
            DACL_SECURITY_INFORMATION,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            needed,
            &mut needed,
        );
        if ok == 0 {
            return Err(os_error("reading the key file DACL"));
        }
        let mut rendered = std::ptr::null_mut();
        let ok = ConvertSecurityDescriptorToStringSecurityDescriptorW(
            buffer.as_ptr() as *const std::ffi::c_void,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut rendered,
            std::ptr::null_mut(),
        );
        if ok == 0 || rendered.is_null() {
            return Err(os_error("rendering the key file DACL"));
        }
        let mut len = 0usize;
        while *rendered.add(len) != 0 {
            len += 1;
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(rendered, len));
        LocalFree(rendered as *mut std::ffi::c_void);
        Ok(text)
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

#[cfg(windows)]
fn os_error(what: &str) -> anyhow::Error {
    anyhow::anyhow!("{what}: {}", std::io::Error::last_os_error())
}

#[cfg(windows)]
#[allow(unsafe_code)]
extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        string_security_descriptor: *const u16,
        string_sd_revision: u32,
        security_descriptor: *mut *mut std::ffi::c_void,
        security_descriptor_size: *mut u32,
    ) -> i32;

    fn ConvertSecurityDescriptorToStringSecurityDescriptorW(
        security_descriptor: *const std::ffi::c_void,
        requested_string_sd_revision: u32,
        security_information: u32,
        string_security_descriptor: *mut *mut u16,
        string_security_descriptor_len: *mut u32,
    ) -> i32;

    fn GetFileSecurityW(
        file_name: *const u16,
        security_information: u32,
        security_descriptor: *mut std::ffi::c_void,
        length: u32,
        length_needed: *mut u32,
    ) -> i32;

    fn SetKernelObjectSecurity(
        handle: *mut std::ffi::c_void,
        security_information: u32,
        security_descriptor: *const std::ffi::c_void,
    ) -> i32;

    fn LocalFree(memory: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_only_dacl_rejects_everyone_users_and_a_null_dacl() {
        assert!(dacl_is_owner_only("D:P(A;;FA;;;OW)"));
        assert!(dacl_is_owner_only(
            "D:(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;S-1-5-21-1-2-3-1001)"
        ));
        assert!(!dacl_is_owner_only("D:(A;;FA;;;WD)"));
        assert!(!dacl_is_owner_only("D:(A;;FA;;;BU)"));
        assert!(!dacl_is_owner_only("D:(A;;FA;;;AU)"));
        assert!(!dacl_is_owner_only("D:(A;;FA;;;S-1-1-0)"));
        assert!(!dacl_is_owner_only("D:(A;;FA;;;S-1-5-11)"));
        assert!(!dacl_is_owner_only("D:(A;;FA;;;S-1-5-32-545)"));
        assert!(!dacl_is_owner_only("D:NO_ACCESS_CONTROL"));
        assert!(!dacl_is_owner_only(""));
        // A deny ACE for Everyone does not by itself grant access.
        assert!(dacl_is_owner_only("D:P(D;;FA;;;WD)(A;;FA;;;OW)"));
    }

    #[test]
    fn owner_only_check_rejects_a_group_readable_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seed.txt");
        fs::write(&path, b"00").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            let error = ensure_owner_only(&path).unwrap_err().to_string();
            assert!(error.contains("other users"), "{error}");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            assert!(ensure_owner_only(&path).is_ok());
        }
        #[cfg(windows)]
        {
            let file = fs::OpenOptions::new().write(true).open(&path).unwrap();
            tighten_private_file(&file).unwrap();
            drop(file);
            assert!(ensure_owner_only(&path).is_ok());
        }
    }
}
