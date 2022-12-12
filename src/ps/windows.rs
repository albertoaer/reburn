use std::{process::{Command, Child}, ffi::OsStr, io, collections::LinkedList, mem};
use winapi::shared::ntdef::HANDLE;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, TH32CS_SNAPPROCESS, PROCESSENTRY32};
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE};

#[inline]
fn open_process(id: u32) -> io::Result<HANDLE> {
  let handle = unsafe {
    OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, 0, id)
  };
  if handle.is_null() {
    return Err(io::Error::new(io::ErrorKind::NotFound, "Handle process error"))
  }
  return Ok(handle)
}

#[inline]
fn kill_process(handle: HANDLE) -> io::Result<()> {
  if
    unsafe { TerminateProcess(handle, 1) } == 0 ||
    unsafe { CloseHandle(handle) } == 0
  {
    return Err(io::Error::new(io::ErrorKind::Other, "Error terminating process"))
  }
  Ok(())
}

fn process_snapshot() -> LinkedList<(u32, u32)> {
  let mut idlist: LinkedList<(u32, u32)> = LinkedList::new(); // (ppid, pid)
  unsafe {
    let mut entry: PROCESSENTRY32 = mem::zeroed();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if Process32First(snap, &mut entry) == 0 {
      return idlist;
    }
    loop {
      if entry.th32ParentProcessID > 4 || entry.th32ProcessID > 4 {
        idlist.push_back((entry.th32ParentProcessID, entry.th32ProcessID));
      }
      if Process32Next(snap, &mut entry) == 0 {
        break
      }
    }
  }
  return idlist;
}

pub struct Process(Child, HANDLE);

impl Process {
	pub fn run<S : AsRef<OsStr>>(args: &[S]) -> io::Result<Self> {
		let child = Command::new(&args[0]).args(&args[1..]).spawn()?;
		let handle = open_process(child.id())?;
		Ok(Process(child, handle))
	}

	pub fn kill(&mut self) {
    let idlist = process_snapshot();
		let mut parents: Vec<u32> = vec![self.0.id()];
		while let Some(current) = parents.pop() {
			for id in idlist.iter() {
				if id.0 == current {
          open_process(id.1).and_then(kill_process).ok();
					parents.push(id.1)
				}
			}
		}
		kill_process(self.1).ok();
	}
}