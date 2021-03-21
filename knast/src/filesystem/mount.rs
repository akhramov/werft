/// Bindings around mount and umount(2) syscalls.
use std::{convert::AsRef, io::Error as StdError, io::IoSlice, path::Path};

use anyhow::{anyhow, Error};

#[fehler::throws]
pub fn mount<'a>(
    kind: &dyn AsRef<Path>,
    source: &dyn AsRef<Path>,
    destination: &dyn AsRef<Path>,
    options: impl Iterator<Item = &'a dyn AsRef<str>>,
) {
    let kind = kind.as_bytes()?;
    let source = source.as_bytes()?;
    let destination = destination.as_bytes()?;

    let mut iovecs = vec![
        IoSlice::new(b"fstype\0"),
        IoSlice::new(kind.as_slice()),
        IoSlice::new(b"fspath\0"),
        IoSlice::new(destination.as_slice()),
        IoSlice::new(b"from\0"),
        IoSlice::new(source.as_slice()),
        IoSlice::new(b"errmsg\0"),
        IoSlice::new(&[0; 255]),
    ];

    for key in options {
        for option in key.as_ref().split("=") {
            iovecs.push(IoSlice::new(option.as_bytes()));
        }
    }

    let slice = iovecs.as_slice();

    if unsafe { libc::nmount(slice as *const _ as _, iovecs.len() as _, 0) }
        < 0
    {
        fehler::throw!(anyhow!(
            "mount: nmount failed: {}",
            StdError::last_os_error()
        ))
    };
}

#[fehler::throws]
pub fn unmount(destination: &dyn AsRef<Path>) {
    if unsafe {
        libc::unmount(destination.as_bytes()?.as_slice() as *const _ as _, 0)
    } < 0
    {
        fehler::throw!(anyhow!(
            "mount: unmount failed: {}",
            StdError::last_os_error(),
        ))
    }
}

trait AsBytes {
    #[fehler::throws]
    fn as_bytes(&self) -> Vec<u8>;
}

impl AsBytes for &dyn AsRef<Path> {
    // TODO: too complex. Is there a better way?
    #[fehler::throws]
    fn as_bytes(&self) -> Vec<u8> {
        use std::{ffi::CString, ffi::OsStr, os::unix::ffi::OsStrExt};

        let path: &Path = self.as_ref();
        let os_str: &OsStr = path.as_ref();
        CString::new(os_str.as_bytes())?.into_bytes_with_nul()
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_mounting_nullfs() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();

        mount(&"nullfs", &source.path(), &dest.path(), std::iter::empty())
            .expect("failed to mount nullfs");

        let mount_output = Command::new("mount")
            .output()
            .expect("Failed to execute mount");

        let output_string = String::from_utf8(mount_output.stdout).unwrap();

        assert!(output_string.contains(&format!(
            "{} on {} (nullfs",
            source.path().display(),
            dest.path().display()
        )));

        unmount(&dest.path()).expect("failed to unmount nullfs");
    }
}