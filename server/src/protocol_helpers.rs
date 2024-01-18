use byteorder::{ByteOrder, LittleEndian};
use nix::errno::Errno::EINTR;
use nix::sys::socket::recv;
use nix::sys::socket::MsgFlags;
use std::convert::TryInto;
use std::mem::size_of;
use std::os::unix::io::RawFd;

pub fn recv_u64(fd: RawFd) -> Result<u64, String> {
    let mut buf = [0u8; size_of::<u64>()];
    recv_loop(fd, &mut buf, size_of::<u64>().try_into().unwrap()).unwrap();
    let val = LittleEndian::read_u64(&buf);
    Ok(val)
}

/// Receive `len` bytes from a connection-orriented socket
pub fn recv_loop(fd: RawFd, buf: &mut [u8], len: u64) -> Result<(), String> {
    let len: usize = len.try_into().map_err(|err| format!("{:?}", err)).unwrap();
    let mut recv_bytes = 0;

    while recv_bytes < len {
        let size = match recv(fd, &mut buf[recv_bytes..len], MsgFlags::empty()) {
            Ok(size) => size,
            Err(nix::Error::Sys(EINTR)) => 0,
            Err(err) => return Err(format!("{:?}", err)),
        };
        recv_bytes += size;
    }

    Ok(())
}
