use std::path::PathBuf;

use nt_tui::FolderPickerError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DialogUnavailable;

#[cfg(not(windows))]
pub(crate) fn choose_epub() -> Result<Option<PathBuf>, DialogUnavailable> {
    Err(DialogUnavailable)
}

#[cfg(not(windows))]
pub(crate) fn choose_folder() -> Result<Option<PathBuf>, FolderPickerError> {
    Err(FolderPickerError::Unavailable)
}

#[cfg(windows)]
pub(crate) use windows::{choose_epub, choose_folder};

#[cfg(windows)]
mod windows {
    use super::{DialogUnavailable, FolderPickerError, PathBuf};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr::{NonNull, null_mut};
    use std::sync::Mutex;
    use winapi::Interface;
    use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
    use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize};
    use winapi::um::objbase::COINIT_APARTMENTTHREADED;
    use winapi::um::shobjidl::{
        FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
        FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, IFileOpenDialog,
    };
    use winapi::um::shobjidl_core::{CLSID_FileOpenDialog, IShellItem, SIGDN_FILESYSPATH};
    use winapi::um::shtypes::COMDLG_FILTERSPEC;
    use winapi::um::unknwnbase::IUnknown;

    const CANCELLED: i32 = 0x8007_04c7_u32 as i32;
    const ACCESS_DENIED: i32 = 0x8007_0005_u32 as i32;
    static ACTIVE_DIALOG: Mutex<()> = Mutex::new(());

    enum Selection {
        Epub,
        Folder,
    }

    struct Apartment;

    impl Drop for Apartment {
        fn drop(&mut self) {
            // Drops on the same OS thread that initialized COM.
            unsafe { CoUninitialize() }
        }
    }

    struct ComInterface<T>(NonNull<T>);

    impl<T> Drop for ComInterface<T> {
        fn drop(&mut self) {
            // The only inputs to this owner are borrowed from documented COM
            // out parameters returning one owned interface reference.
            unsafe { self.0.cast::<IUnknown>().as_ref().Release() };
        }
    }

    struct DisplayName(NonNull<u16>);

    impl Drop for DisplayName {
        fn drop(&mut self) {
            // IShellItem::GetDisplayName allocates this string with COM's allocator.
            unsafe { CoTaskMemFree(self.0.as_ptr().cast()) };
        }
    }

    fn check(result: i32) -> Result<(), FolderPickerError> {
        match result {
            ACCESS_DENIED => Err(FolderPickerError::PermissionDenied),
            value if value < 0 => Err(FolderPickerError::Unavailable),
            _ => Ok(()),
        }
    }

    pub(crate) fn choose_epub() -> Result<Option<PathBuf>, DialogUnavailable> {
        choose(Selection::Epub).map_err(|_| DialogUnavailable)
    }

    pub(crate) fn choose_folder() -> Result<Option<PathBuf>, FolderPickerError> {
        choose(Selection::Folder)
    }

    fn choose(selection: Selection) -> Result<Option<PathBuf>, FolderPickerError> {
        let _active = ACTIVE_DIALOG
            .try_lock()
            .map_err(|_| FolderPickerError::Busy)?;
        // This function is called on a dedicated worker. All interfaces, filter
        // buffers and the returned string outlive their calls and are released
        // before leaving the same single-threaded apartment.
        unsafe {
            check(CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED))?;
            let _apartment = Apartment;
            let mut object = null_mut();
            check(CoCreateInstance(
                &CLSID_FileOpenDialog,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IFileOpenDialog::uuidof(),
                &mut object,
            ))?;
            let dialog = ComInterface(
                NonNull::new(object.cast::<IFileOpenDialog>())
                    .ok_or(FolderPickerError::Unavailable)?,
            );
            let picker = dialog.0.as_ref();
            let name: Vec<u16> = "EPUB\0".encode_utf16().collect();
            let pattern: Vec<u16> = "*.epub\0".encode_utf16().collect();
            let filter = COMDLG_FILTERSPEC {
                pszName: name.as_ptr(),
                pszSpec: pattern.as_ptr(),
            };
            let flags = FOS_PATHMUSTEXIST
                | FOS_FORCEFILESYSTEM
                | FOS_NOCHANGEDIR
                | match selection {
                    Selection::Epub => FOS_FILEMUSTEXIST,
                    Selection::Folder => FOS_PICKFOLDERS,
                };
            let mut options = 0;
            check(picker.GetOptions(&mut options))?;
            check(picker.SetOptions((options | flags) & !FOS_ALLOWMULTISELECT))?;
            if matches!(selection, Selection::Epub) {
                check(picker.SetFileTypes(1, &filter))?;
            }
            let shown = picker.Show(null_mut());
            if shown == CANCELLED {
                return Ok(None);
            }
            check(shown)?;
            let mut selected: *mut IShellItem = null_mut();
            check(picker.GetResult(&mut selected))?;
            let selected =
                ComInterface(NonNull::new(selected).ok_or(FolderPickerError::Unavailable)?);
            let mut display = null_mut();
            check(
                selected
                    .0
                    .as_ref()
                    .GetDisplayName(SIGDN_FILESYSPATH, &mut display),
            )?;
            let display = DisplayName(NonNull::new(display).ok_or(FolderPickerError::Unavailable)?);
            let path = display.0.as_ptr();
            let length = (0..32_768)
                .find(|&index| *path.add(index) == 0)
                .ok_or(FolderPickerError::Unavailable)?;
            if length == 0 {
                return Err(FolderPickerError::Unavailable);
            }
            Ok(Some(PathBuf::from(OsString::from_wide(
                std::slice::from_raw_parts(path, length),
            ))))
        }
    }
}
