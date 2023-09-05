use std::{net::TcpStream, io::{Write, Read}};

use anyhow::Result;

use crate::{utils::AsByteEncoded, Header};

pub trait Transport {
    fn new(url: String) -> Self;

    fn send_without_response(&self, data: impl AsByteEncoded) -> Result<()>;

    fn send_with_response<T: Copy>(&self, data: impl AsByteEncoded) -> Result<T>;
}

pub struct Tcp {
    url: String,
}

impl Transport for Tcp {
    fn new(url: String) -> Self {
        Self {
            url
        }
    }

    fn send_without_response(&self, data: impl AsByteEncoded) -> Result<()> {
        let mut stream = TcpStream::connect(&self.url)?;

        stream.write_all(data.encode_as_bytes())?;

        Ok(())
    }

    fn send_with_response<T: Copy>(&self, data: impl AsByteEncoded) -> Result<T> {
        let mut stream = TcpStream::connect(&self.url)?;

        let mut buf = vec![0; std::mem::size_of::<T>() + 100];
        stream.write_all(data.encode_as_bytes())?;

        stream.read(&mut buf)?;

        let header = unsafe { *(buf.as_ptr().offset(24) as *const Header) };

        buf.truncate(24 + header.get_size());

        let res = unsafe {
            *(buf.as_ptr().offset(24 + std::mem::size_of::<Header>() as isize) as *const T)
        };

        Ok(res)
    }
}