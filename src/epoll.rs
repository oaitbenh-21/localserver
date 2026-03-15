// src/epoll.rs

use libc::{
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLLET, EPOLLIN, epoll_create1, epoll_ctl, epoll_event,
    epoll_wait,
};
use std::os::unix::io::RawFd;

pub const MAX_EVENTS: usize = 128;

#[derive(Debug)]
pub struct Epoll {
    fd: RawFd,
}

impl Epoll {
    pub fn new() -> Result<Epoll, std::io::Error> {
        let fd = unsafe { epoll_create1(0) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Epoll { fd })
    }

    pub fn add(&self, socket_fd: RawFd) -> Result<(), std::io::Error> {
        let mut event = epoll_event {
            // EPOLLIN  = we want to know when data arrives
            // EPOLLET  = edge-triggered mode
            events: (EPOLLIN | EPOLLET) as u32,
            u64: socket_fd as u64, // store the fd so we know who fired
        };

        let result = unsafe { epoll_ctl(self.fd, EPOLL_CTL_ADD, socket_fd, &mut event) };

        if result < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn remove(&self, socket_fd: RawFd) -> Result<(), std::io::Error> {
        let result = unsafe { epoll_ctl(self.fd, EPOLL_CTL_DEL, socket_fd, std::ptr::null_mut()) };

        if result < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn wait(
        &self,
        events: &mut [epoll_event],
        timeout_ms: i32,
    ) -> Result<usize, std::io::Error> {
        let result = unsafe {
            epoll_wait(
                self.fd,
                events.as_mut_ptr(),
                events.len() as i32,
                timeout_ms, // -1 means block forever until something is ready
            )
        };

        if result < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(result as usize)
    }
}

// When Epoll is dropped, close the file descriptor
impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

// add this at the top with the other use statements
use libc::{F_GETFL, F_SETFL, O_NONBLOCK, fcntl};
// fcntl  read and modify  settings or fd.
// The name fcntl literally means "file control". Same pattern as epoll_ctl
// F_GETFL  →  GET the FLags currently set on this fd
// F_SETFL  →  SET the FLags on this fd
pub fn set_nonblocking(fd: RawFd) -> Result<(), std::io::Error> {
    let flags = unsafe { fcntl(fd, F_GETFL, 0) }; // step 1: read current flags
    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }

    let result = unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) }; // step 2: write them back with O_NONBLOCK added
    if result < 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}
