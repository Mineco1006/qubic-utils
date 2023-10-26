
#[cfg(not(any(feature = "async", feature = "http")))]
use std::{net::TcpStream, io::{Write, Read}};

use anyhow::Result;

use qubic_tcp_types::{Header, types::{Packet, ExchangePublicPeers}, MessageType, utils::QubicRequest};
use qubic_types::traits::AsByteEncoded;

#[cfg(any(feature = "async", feature = "http"))]
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[cfg(not(any(feature = "async", feature = "http")))]
pub trait Transport {
    fn new(url: String) -> Self;

    fn send_without_response(&self, data: impl AsByteEncoded) -> Result<()>;

    fn send_with_response<T: Copy, D: QubicRequest>(&self, data: Packet<D>) -> Result<T>;

    fn is_connected(&self) -> bool;
}

#[cfg(any(feature = "async", feature = "http"))]
#[async_trait::async_trait]
pub trait Transport {
    async fn new(url: String) -> Self;

    async fn send_without_response<B: AsByteEncoded + Send>(&self, data: B) -> Result<()>;

    async fn send_with_response<T: Copy, D: QubicRequest + Send>(&self, data: Packet<D>) -> Result<T>;

    async fn is_connected(&self) -> bool;
}

pub struct Tcp {
    pub(crate) url: String,
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

    async fn send_with_response<T: Copy, D: QubicRequest + Send>(&self, data: Packet<D>) -> Result<T> {
        use tokio::net::TcpStream;
        let mut stream = TcpStream::connect(&self.url).await?;

        let mut buf = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>() + std::mem::size_of::<Packet<T>>()];
        stream.write_all(data.encode_as_bytes()).await?;

        stream.read_exact(&mut buf).await?;

        let header = unsafe { *(buf.as_ptr() as *const Header) };

        let offset = if header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers { std::mem::size_of::<Packet<ExchangePublicPeers>>() as isize } else { 0 };

        let res = unsafe {
            std::ptr::read_unaligned(buf.as_ptr().offset(offset + std::mem::size_of::<Header>() as isize) as *const T)
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
            url,
            is_connected: false
        }
    }

    fn send_without_response(&self, data: impl AsByteEncoded) -> Result<()> {
        let mut stream = TcpStream::connect(&self.url)?;

        stream.write_all(data.encode_as_bytes())?;

        Ok(())
    }

    fn send_with_response<T: Copy, D: QubicRequest>(&self, data: Packet<D>) -> Result<T> {
        let mut stream = TcpStream::connect(&self.url)?;

        let mut buf = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>() + std::mem::size_of::<Packet<T>>()];
        stream.write_all(data.encode_as_bytes())?;

        stream.read_exact(&mut buf)?;

        let header = unsafe { *(buf.as_ptr() as *const Header) };

        let offset = if header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers { std::mem::size_of::<Packet<ExchangePublicPeers>>() as isize } else { 0 };

        let res = unsafe {
            std::ptr::read_unaligned(buf.as_ptr().offset(offset + std::mem::size_of::<Header>() as isize) as *const T)
        };

        Ok(res)
    }

    fn is_connected(&self) -> bool {
        false
    }
}