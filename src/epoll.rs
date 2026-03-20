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

#[cfg(test)]
mod tests {

    use super::*;
    use std::net::TcpListener;
    use std::os::unix::io::AsRawFd;

    // ── Epoll creation ────────────────────────────────────────────────────

    #[test]
    fn test_epoll_creates_successfully() {
        let epoll = Epoll::new();
        assert!(epoll.is_ok());
    }

    #[test]
    fn test_epoll_fd_is_valid() {
        let epoll = Epoll::new().unwrap();
        // A valid file descriptor is always >= 0
        assert!(epoll.fd >= 0);
    }
    // ── Add / Remove ──────────────────────────────────────────────────────

    #[test]
    fn test_add_socket_succeeds() {
        let epoll = Epoll::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let result = epoll.add(listener.as_raw_fd());
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_socket_succeeds() {
        let epoll = Epoll::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let fd = listener.as_raw_fd();

        epoll.add(fd).unwrap();
        let result = epoll.remove(fd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_without_add_fails() {
        let epoll = Epoll::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        // Removing something never added must return an error — not crash
        let result = epoll.remove(listener.as_raw_fd());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_same_fd_twice_fails() {
        let epoll = Epoll::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let fd = listener.as_raw_fd();

        epoll.add(fd).unwrap();

        // Adding the same fd twice must return an error — not crash
        let result = epoll.add(fd);
        assert!(result.is_err());
    }

    // ── Non-blocking ──────────────────────────────────────────────────────

    #[test]
    fn test_set_nonblocking_succeeds() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let result = set_nonblocking(listener.as_raw_fd());
        assert!(result.is_ok());
    }

    #[test]
    fn test_socket_is_actually_nonblocking() {
        use std::io::Read;
        use std::net::TcpStream;

        // Create a real socket and make it non-blocking
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        let fd = stream.as_raw_fd();

        set_nonblocking(fd).unwrap();

        // On a non-blocking socket with no data, read must return
        // WouldBlock immediately instead of freezing
        let mut buf = [0u8; 1];
        let mut raw_stream = unsafe {
            use std::os::unix::io::FromRawFd;
            TcpStream::from_raw_fd(fd)
        };

        match raw_stream.read(&mut buf) {
            Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::WouldBlock),
            Ok(_) => {} // connection might have data, that's fine too
        }

        std::mem::forget(raw_stream); // don't double-close
    }

    #[test]
    fn test_epoll_drop_closes_fd() {
        let fd = {
            let epoll = Epoll::new().unwrap();
            epoll.fd
        }; // epoll dropped here — fd should be closed

        // Trying to use a closed fd must fail
        let result = unsafe { libc::epoll_wait(fd, std::ptr::null_mut(), 0, 0) };
        assert_eq!(result, -1); // -1 means the fd is invalid
    }
}
