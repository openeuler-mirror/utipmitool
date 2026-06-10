use nix::errno::Errno;
use std::mem;
use std::os::fd::RawFd;

const IPMI_IOC_MAGIC: u8 = b'i';

const IPMICTL_RECEIVE_MSG_TRUNC: u8 = 11;
const IPMICTL_SEND_COMMAND: u8 = 13;
const IPMICTL_SET_GETS_EVENTS_CMD: u8 = 16;
const IPMICTL_SET_MY_ADDRESS_CMD: u8 = 17;

// Linux ioctl encoding (asm-generic/ioctl.h)
const IOC_NRBITS: u8 = 8;
const IOC_TYPEBITS: u8 = 8;
const IOC_SIZEBITS: u8 = 14;
const IOC_DIRBITS: u8 = 2;

const IOC_NRSHIFT: u8 = 0;
const IOC_TYPESHIFT: u8 = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: u8 = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: u8 = IOC_SIZESHIFT + IOC_SIZEBITS;

const IOC_NONE: u8 = 0;
const IOC_WRITE: u8 = 1;
const IOC_READ: u8 = 2;

fn ioc(dir: u8, type_: u8, nr: u8, size: u16) -> libc::c_ulong {
    let dir_part = (dir as libc::c_ulong) << IOC_DIRSHIFT;
    let type_part = (type_ as libc::c_ulong) << IOC_TYPESHIFT;
    let nr_part = (nr as libc::c_ulong) << IOC_NRSHIFT;
    let size_part = (size as libc::c_ulong) << IOC_SIZESHIFT;
    dir_part | type_part | nr_part | size_part
}

fn ioctl_with<T>(fd: RawFd, request: libc::c_ulong, arg: &mut T) -> Result<i32, Errno> {
    let rc = unsafe { libc::ioctl(fd, request, arg as *mut T) };
    if rc < 0 {
        Err(Errno::last())
    } else {
        Ok(rc)
    }
}

pub fn receive_msg_trunc<T>(fd: RawFd, recv: &mut T) -> Result<i32, Errno> {
    let size = mem::size_of::<T>() as u16;
    let request = ioc(
        IOC_READ | IOC_WRITE,
        IPMI_IOC_MAGIC,
        IPMICTL_RECEIVE_MSG_TRUNC,
        size,
    );
    ioctl_with(fd, request, recv)
}

pub fn send_command<T>(fd: RawFd, req: &mut T) -> Result<i32, Errno> {
    let size = mem::size_of::<T>() as u16;
    let request = ioc(IOC_READ, IPMI_IOC_MAGIC, IPMICTL_SEND_COMMAND, size);
    ioctl_with(fd, request, req)
}

pub fn set_get_events_cmd(fd: RawFd, receive_events: &mut i32) -> Result<i32, Errno> {
    let size = mem::size_of::<i32>() as u16;
    let request = ioc(IOC_READ, IPMI_IOC_MAGIC, IPMICTL_SET_GETS_EVENTS_CMD, size);
    ioctl_with(fd, request, receive_events)
}

pub fn set_my_address_cmd(fd: RawFd, addr: &mut u32) -> Result<i32, Errno> {
    let size = mem::size_of::<u32>() as u16;
    let request = ioc(IOC_READ, IPMI_IOC_MAGIC, IPMICTL_SET_MY_ADDRESS_CMD, size);
    ioctl_with(fd, request, addr)
}

#[allow(dead_code)]
fn _assert_constants() {
    let _ = IOC_NONE;
    let _ = IOC_DIRBITS;
}
