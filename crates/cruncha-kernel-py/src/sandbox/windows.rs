// SPDX-License-Identifier: GPL-3.0-or-later
//! The worker's sandbox on Windows (ADR-0030 §3): an AppContainer with no capabilities inside
//! a Job Object.
//!
//! - **AppContainer** `io.github.acmeanvil.cruncha.worker`: no network (no `internetClient`
//!   or loopback), and no files except those readable by every app package (system
//!   libraries, fonts) and those [`prepare`] grants to its SID: the readable trees
//!   (environment, base interpreter, worker package: read and execute) and the scratch
//!   directory (full access). Grants are inheritable and set once per tree.
//! - **Job Object**: an active-process limit (one, or two when the interpreter is a venv's
//!   redirector, which starts the real interpreter as its child), so the worker can't start
//!   programs; kill-on-close, so the worker dies with the launcher; no clipboard, desktop or
//!   other processes' window handles; no error dialogs.
//!
//! [`launch`] is the launcher's `main`: it runs `<python> <args…>` that way with its own
//! stdin, stdout and stderr (and nothing else) inherited, waits, and exits with its code.
//! Killing the launcher closes the job, which kills the worker.

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, HANDLE_FLAG_INHERIT, HLOCAL, INVALID_HANDLE_VALUE, LocalFree,
    SetHandleInformation,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertStringSidToSidW, EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW, NO_MULTIPLE_TRUSTEE,
    SE_FILE_OBJECT, SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP,
    TRUSTEE_W,
};
use windows_sys::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeriveAppContainerSidFromAppContainerName,
};
use windows_sys::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, DACL_SECURITY_INFORMATION, EqualSid, FreeSid, GetAce, PSECURITY_DESCRIPTOR,
    PSID, SECURITY_CAPABILITIES, SUB_CONTAINERS_AND_OBJECTS_INHERIT,
};
use windows_sys::Win32::Storage::FileSystem::{FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ};
use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_UILIMIT_DESKTOP,
    JOB_OBJECT_UILIMIT_DISPLAYSETTINGS, JOB_OBJECT_UILIMIT_EXITWINDOWS, JOB_OBJECT_UILIMIT_GLOBALATOMS,
    JOB_OBJECT_UILIMIT_HANDLES, JOB_OBJECT_UILIMIT_READCLIPBOARD, JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS,
    JOB_OBJECT_UILIMIT_WRITECLIPBOARD, JOBOBJECT_BASIC_UI_RESTRICTIONS, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectBasicUIRestrictions, JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{
    CREATE_NO_WINDOW, CREATE_SUSPENDED, CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
    GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, PROCESS_INFORMATION,
    ResumeThread, STARTF_USESTDHANDLES, STARTUPINFOEXW, TerminateProcess, UpdateProcThreadAttribute,
    WaitForSingleObject,
};

use super::{Sandbox, push_arg, venv_home};

/// The AppContainer's name (its profile lives in `%LOCALAPPDATA%\Packages\<name>`).
pub const CONTAINER: &str = "io.github.acmeanvil.cruncha.worker";

/// `HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS)`.
const ALREADY_EXISTS: i32 = 0x8007_00B7_u32 as i32;

/// ALL APPLICATION PACKAGES: what every AppContainer may read (system files, Program Files).
const ALL_APP_PACKAGES: &str = "S-1-15-2-1";

fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}

fn last_error(what: &str) -> String {
    // SAFETY: reads this thread's last error code.
    format!("{what} failed (error {})", unsafe { GetLastError() })
}

/// A SID freed on drop (`FreeSid` for AppContainer SIDs, `LocalFree` for converted ones).
struct Sid {
    ptr: PSID,
    local: bool,
}

impl Drop for Sid {
    fn drop(&mut self) {
        // SAFETY: the pointer came from the matching allocator and is freed once.
        unsafe {
            if self.local {
                LocalFree(self.ptr as HLOCAL);
            } else {
                FreeSid(self.ptr);
            }
        }
    }
}

/// The worker's AppContainer SID, creating its profile the first time.
fn container_sid() -> Result<Sid, String> {
    let name = wide(OsStr::new(CONTAINER));
    let display = wide(OsStr::new("Cruncha Python worker"));
    let mut sid: PSID = null_mut();
    // SAFETY: valid NUL-terminated strings; no capabilities; the SID is returned in `sid`.
    let mut hr = unsafe { CreateAppContainerProfile(name.as_ptr(), display.as_ptr(), display.as_ptr(), null(), 0, &mut sid) };
    if hr == ALREADY_EXISTS {
        // SAFETY: as above.
        hr = unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid) };
    }
    if hr < 0 || sid.is_null() {
        return Err(format!("couldn't create the worker's AppContainer (HRESULT {hr:#010x})"));
    }
    Ok(Sid { ptr: sid, local: false })
}

fn well_known(s: &str) -> Result<Sid, String> {
    let w = wide(OsStr::new(s));
    let mut sid: PSID = null_mut();
    // SAFETY: a valid SID string; the result is LocalAlloc'd.
    if unsafe { ConvertStringSidToSidW(w.as_ptr(), &mut sid) } == 0 {
        return Err(last_error("ConvertStringSidToSidW"));
    }
    Ok(Sid { ptr: sid, local: true })
}

/// Whether `dacl` already gives one of `sids` `mask`, inherited by everything below.
///
/// # Safety
/// `dacl` is null or a valid ACL.
unsafe fn allows(dacl: *const ACL, sids: &[&Sid], mask: u32) -> bool {
    if dacl.is_null() {
        return true; // no DACL: everyone has access
    }
    // SAFETY: `dacl` is valid; `GetAce` returns pointers into it; an ACCESS_ALLOWED ACE's SID
    // starts at `SidStart`.
    unsafe {
        for i in 0..u32::from((*dacl).AceCount) {
            let mut ace: *mut core::ffi::c_void = null_mut();
            if GetAce(dacl, i, &mut ace) == 0 {
                continue;
            }
            let header = &*(ace as *const ACE_HEADER);
            if header.AceType != 0 || u32::from(header.AceFlags) & 3 != 3 {
                continue; // not ACCESS_ALLOWED, or not inherited by files and folders
            }
            let allowed = &*(ace as *const ACCESS_ALLOWED_ACE);
            let sid = &allowed.SidStart as *const u32 as PSID;
            if allowed.Mask & mask == mask && sids.iter().any(|s| EqualSid(sid, s.ptr) != 0) {
                return true;
            }
        }
    }
    false
}

/// Give `sid` `mask` on `path` and everything below it, unless `path` already allows it to
/// one of `already`. Setting the DACL propagates the inheritable entry to existing children.
fn grant(path: &Path, sid: &Sid, already: &[&Sid], mask: u32) -> Result<(), String> {
    let w = wide(path.as_os_str());
    let mut dacl: *mut ACL = null_mut();
    let mut sd: PSECURITY_DESCRIPTOR = null_mut();
    let why = |what: &str, e: u32| format!("{what} on {}: error {e}", path.display());
    // SAFETY: `w` is NUL-terminated; outputs are valid pointers; `sd` is freed below and
    // `dacl` points into it.
    unsafe {
        let e = GetNamedSecurityInfoW(
            w.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut dacl,
            null_mut(),
            &mut sd,
        );
        if e != 0 {
            return Err(why("reading permissions", e));
        }
        if allows(dacl, already, mask) {
            LocalFree(sd as HLOCAL);
            return Ok(());
        }
        let entry = EXPLICIT_ACCESS_W {
            grfAccessPermissions: mask,
            grfAccessMode: GRANT_ACCESS,
            grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: null_mut(),
                MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
                ptstrName: sid.ptr as *mut u16,
            },
        };
        let mut new: *mut ACL = null_mut();
        let e = SetEntriesInAclW(1, &entry, dacl, &mut new);
        LocalFree(sd as HLOCAL);
        if e != 0 {
            return Err(why("building permissions", e));
        }
        let e = SetNamedSecurityInfoW(w.as_ptr(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION, null_mut(), null_mut(), new, null());
        LocalFree(new as HLOCAL);
        if e != 0 {
            return Err(why("granting the worker access", e));
        }
    }
    Ok(())
}

/// Before a sandboxed start: create the scratch directory and grant the worker's
/// AppContainer the readable trees (plus a venv's base interpreter) and the scratch
/// directory. The first grant on a large environment takes a few seconds; later ones only
/// read the top directory's permissions.
pub fn prepare(sandbox: &Sandbox, python: &Path) -> Result<(), String> {
    std::fs::create_dir_all(&sandbox.scratch).map_err(|e| format!("{}: {e}", sandbox.scratch.display()))?;
    let sid = container_sid()?;
    let everyone_app = well_known(ALL_APP_PACKAGES)?;
    let mut read: Vec<PathBuf> = sandbox.read.clone();
    read.extend(venv_home(python));
    for p in &read {
        if p.exists() {
            grant(p, &sid, &[&sid, &everyone_app], FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
        }
    }
    grant(&sandbox.scratch, &sid, &[&sid], FILE_ALL_ACCESS)
}

/// The launcher's `main`: `args` are the interpreter and its arguments. Returns the worker's
/// exit code, or 125 when the sandbox couldn't be set up (the reason goes to stderr).
pub fn launch(args: Vec<OsString>) -> i32 {
    match run(&args) {
        Ok(code) => code as i32,
        Err(e) => {
            eprintln!("cruncha: the Python worker's sandbox: {e}");
            125
        }
    }
}

fn run(args: &[OsString]) -> Result<u32, String> {
    let python = PathBuf::from(args.first().ok_or("no interpreter given")?);
    let sid = container_sid()?;
    // A venv's python.exe is a redirector with the real interpreter as its one child.
    let processes = if venv_home(&python).is_some() { 2 } else { 1 };

    let mut line = Vec::new();
    for a in args {
        push_arg(&mut line, &a.encode_wide().collect::<Vec<_>>());
    }
    line.push(0);
    let program = wide(python.as_os_str());

    // SAFETY: every pointer passed below is valid for the call; handles are closed and the
    // attribute list deleted before returning (the job's handle closes when this process
    // exits, which kills whatever is left in it).
    unsafe {
        let job = CreateJobObjectW(null(), null());
        if job.is_null() {
            return Err(last_error("CreateJobObjectW"));
        }
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
        limits.BasicLimitInformation.ActiveProcessLimit = processes;
        if SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const core::ffi::c_void,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(last_error("limiting the worker's job"));
        }
        let ui = JOBOBJECT_BASIC_UI_RESTRICTIONS {
            UIRestrictionsClass: JOB_OBJECT_UILIMIT_HANDLES
                | JOB_OBJECT_UILIMIT_READCLIPBOARD
                | JOB_OBJECT_UILIMIT_WRITECLIPBOARD
                | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS
                | JOB_OBJECT_UILIMIT_DISPLAYSETTINGS
                | JOB_OBJECT_UILIMIT_GLOBALATOMS
                | JOB_OBJECT_UILIMIT_DESKTOP
                | JOB_OBJECT_UILIMIT_EXITWINDOWS,
        };
        if SetInformationJobObject(
            job,
            JobObjectBasicUIRestrictions,
            &ui as *const _ as *const core::ffi::c_void,
            size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
        ) == 0
        {
            return Err(last_error("restricting the worker's job"));
        }

        // The worker inherits this process's standard handles and nothing else.
        let std = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE].map(|n| GetStdHandle(n));
        let mut inherit: Vec<HANDLE> = Vec::new();
        for &h in &std {
            if !h.is_null() && h != INVALID_HANDLE_VALUE && !inherit.contains(&h) {
                SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
                inherit.push(h);
            }
        }

        let mut size = 0usize;
        InitializeProcThreadAttributeList(null_mut(), 2, 0, &mut size);
        let mut buf = vec![0usize; size.div_ceil(size_of::<usize>())];
        let list = buf.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
        if InitializeProcThreadAttributeList(list, 2, 0, &mut size) == 0 {
            return Err(last_error("InitializeProcThreadAttributeList"));
        }
        let caps = SECURITY_CAPABILITIES {
            AppContainerSid: sid.ptr,
            Capabilities: null_mut(),
            CapabilityCount: 0,
            Reserved: 0,
        };
        let mut ok = UpdateProcThreadAttribute(
            list,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            &caps as *const _ as *const core::ffi::c_void,
            size_of::<SECURITY_CAPABILITIES>(),
            null_mut(),
            null(),
        ) != 0;
        if ok && !inherit.is_empty() {
            ok = UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                inherit.as_ptr() as *const core::ffi::c_void,
                inherit.len() * size_of::<HANDLE>(),
                null_mut(),
                null(),
            ) != 0;
        }
        if !ok {
            let e = last_error("UpdateProcThreadAttribute");
            DeleteProcThreadAttributeList(list);
            return Err(e);
        }

        let mut si: STARTUPINFOEXW = std::mem::zeroed();
        si.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        si.StartupInfo.hStdInput = std[0];
        si.StartupInfo.hStdOutput = std[1];
        si.StartupInfo.hStdError = std[2];
        si.lpAttributeList = list;
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        let created = CreateProcessW(
            program.as_ptr(),
            line.as_mut_ptr(),
            null(),
            null(),
            i32::from(!inherit.is_empty()),
            EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED | CREATE_NO_WINDOW,
            null(),
            null(),
            &si.StartupInfo,
            &mut pi,
        ) != 0;
        let e = last_error("starting the worker");
        DeleteProcThreadAttributeList(list);
        if !created {
            return Err(e);
        }
        // In the job before its first instruction runs.
        if AssignProcessToJobObject(job, pi.hProcess) == 0 {
            let e = last_error("putting the worker in its job");
            TerminateProcess(pi.hProcess, 1);
            CloseHandle(pi.hThread);
            CloseHandle(pi.hProcess);
            return Err(e);
        }
        ResumeThread(pi.hThread);
        CloseHandle(pi.hThread);
        WaitForSingleObject(pi.hProcess, INFINITE);
        let mut code = 1u32;
        GetExitCodeProcess(pi.hProcess, &mut code);
        CloseHandle(pi.hProcess);
        Ok(code)
    }
}
