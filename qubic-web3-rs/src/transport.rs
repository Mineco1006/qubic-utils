
#[cfg(not(any(feature = "async", feature = "http")))]
use std::{net::TcpStream, io::{Write, Read}};

use anyhow::Result;

use qubic_tcp_types::Header;
use qubic_types::traits::AsByteEncoded;

#[cfg(any(feature = "async", feature = "http"))]
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[cfg(not(any(feature = "async", feature = "http")))]
pub trait Transport {
    fn new(url: String) -> Self;

    fn send_without_response(&self, data: impl AsByteEncoded) -> Result<()>;

    fn send_with_response<T: Copy>(&self, data: impl AsByteEncoded) -> Result<T>;

    fn is_connected(&self) -> bool;
}

#[cfg(any(feature = "async", feature = "http"))]
#[async_trait::async_trait]
pub trait Transport {
    async fn new(url: String) -> Self;

    async fn send_without_response<B: AsByteEncoded + Send>(&self, data: B) -> Result<()>;

    async fn send_with_response<T: Copy, B: AsByteEncoded + Send>(&self, data: B) -> Result<T>;

    async fn is_connected(&self) -> bool;
}

pub struct Tcp {
    url: String,
}

#[cfg(any(feature = "async", feature = "http"))]
#[async_trait::async_trait]
impl Transport for Tcp {

    async fn new(url: String) -> Self {
        Self {
            url
        }
    }

    async fn send_without_response<B: AsByteEncoded + Send>(&self, data: B) -> Result<()> {
        use tokio::net::TcpStream;

        let mut stream = TcpStream::connect(&self.url).await?;

        stream.write_all(data.encode_as_bytes()).await?;

        Ok(())
    }

    async fn send_with_response<T: Copy, B: AsByteEncoded + Send>(&self, data: B) -> Result<T> {
        use tokio::net::TcpStream;

        let mut stream = TcpStream::connect(&self.url).await?;

        let mut buf = vec![0; std::mem::size_of::<T>() + 100];

        stream.write_all(data.encode_as_bytes()).await?;

        stream.read(&mut buf).await?;

        let offset = if self.is_connected().await { 0 } else { 24 };

        let header = unsafe { *(buf.as_ptr().offset(offset) as *const Header) };

        buf.truncate(offset as usize + header.get_size());

        let res = unsafe {
            *(buf.as_ptr().offset(offset + std::mem::size_of::<Header>() as isize) as *const T)
        };

        Ok(res)
    }

    async fn is_connected(&self) -> bool {
        false
    }
}

#[cfg(not(any(feature = "async", feature = "http")))]
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

        let offset = if self.is_connected() { 0 } else { 24 };

        let header = unsafe { *(buf.as_ptr().offset(offset) as *const Header) };

        buf.truncate(offset as usize + header.get_size());

        let res = unsafe {
            *(buf.as_ptr().offset(offset + std::mem::size_of::<Header>() as isize) as *const T)
        };

        Ok(res)
    }

    fn is_connected(&self) -> bool {
        false
    }
}