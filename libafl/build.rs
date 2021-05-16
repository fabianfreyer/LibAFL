//! special handling to build and link libafl

use rustc_version::{version_meta, Channel};

#[allow(clippy::ptr_arg, clippy::upper_case_acronyms)]
fn main() {
    #[cfg(target_os = "windows")]
    #[allow(clippy::ptr_arg, clippy::upper_case_acronyms)]
    windows::build!(
        
        Windows::Win32::System::SystemServices::{HANDLE, BOOL, PAGE_TYPE, PSTR},
        Windows::Win32::System::Threading::ExitProcess,
        Windows::Win32::System::WindowsProgramming::CloseHandle,
        // API needed for the shared memory
        Windows::Win32::System::SystemServices::{CreateFileMappingA, OpenFileMappingA},
        Windows::Win32::System::Memory::{MapViewOfFile, UnmapViewOfFile},
        Windows::Win32::System::Diagnostics::Debug::{SetUnhandledExceptionFilter, EXCEPTION_POINTERS, EXCEPTION_RECORD, LPTOP_LEVEL_EXCEPTION_FILTER}
    );

    // Set cfg flags depending on release channel
    match version_meta().unwrap().channel {
        Channel::Stable => {
            println!("cargo:rustc-cfg=RUSTC_IS_STABLE");
        }
        Channel::Beta => {
            println!("cargo:rustc-cfg=RUSTC_IS_BETA");
        }
        Channel::Nightly => {
            println!("cargo:rustc-cfg=RUSTC_IS_NIGHTLY");
        }
        Channel::Dev => {
            println!("cargo:rustc-cfg=RUSTC_IS_DEV");
        }
    }
}
