
#[cfg(not(any(feature = "async", feature = "http")))]
use std::{net::TcpStream, io::{Write, Read}};

#[cfg(any(feature = "async", feature = "http"))]
use tokio::net::TcpStream;

use anyhow::{Result, Ok};

use qubic_tcp_types::{Header, types::{Packet, ExchangePublicPeers}, MessageType, utils::QubicRequest};
use qubic_types::traits::AsByteEncoded;

#[cfg(any(feature = "async", feature = "http"))]
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[cfg(not(any(feature = "async", feature = "http")))]
pub trait Transport {
    fn new(url: String) -> Self;

    fn send_without_response(&self, data: impl AsByteEncoded) -> Result<()>;

    fn send_with_response<T: Copy, D: QubicRequest>(&self, data: Packet<D>) -> Result<T>;

    fn send_with_multiple_responses<T: Copy, D: QubicRequest>(&self, data: Packet<D>) -> Result<Vec<T>>;

    fn connect(&self) -> Result<TcpStream>;
}

#[cfg(any(feature = "async", feature = "http"))]
#[async_trait::async_trait]
pub trait Transport {
    async fn new(url: String) -> Self;

    async fn send_without_response<B: AsByteEncoded + Send>(&self, data: B) -> Result<()>;

    async fn send_with_response<T: Copy, D: QubicRequest + Send>(&self, data: Packet<D>) -> Result<T>;

    async fn send_with_multiple_responses<T: Copy + Send, D: QubicRequest + Send>(&self, data: Packet<D>) -> Result<Vec<T>>;

    async fn connect(&self) -> Result<TcpStream>;
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

    async fn send_with_multiple_responses<T: Copy + Send, D: QubicRequest + Send>(&self, data: Packet<D>) -> Result<Vec<T>> {
        use tokio::net::TcpStream;
        let mut ret: Vec<T> = Vec::new();

        let mut stream = TcpStream::connect(&self.url).await?;

        let mut header_buffer = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>()];
        stream.write_all(data.encode_as_bytes()).await?;
        stream.read_exact(&mut header_buffer).await?;
        header_buffer = vec![0; std::mem::size_of::<Header>()];
        let mut data_buffer = vec![0; std::mem::size_of::<T>()];
        

        loop {
            stream.read_exact(&mut header_buffer).await?;

            let header = unsafe { *(header_buffer.as_ptr() as *const Header) };


            if header.message_type == MessageType::EndResponse {
                break;
            }

            
            stream.read_exact(&mut data_buffer).await?;

            let res = unsafe {
                std::ptr::read_unaligned(data_buffer.as_ptr() as *const T)
            };

            ret.push(res);

            let size = header.get_size() - std::mem::size_of::<Header>() - std::mem::size_of::<T>();

            if size > 0 {
                stream.read_exact(&mut vec![0; size]).await?;
            }
        }
        
        Ok(ret)
    }

    async fn connect(&self) -> Result<TcpStream> {
        Ok(TcpStream::connect(&self.url).await?)
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

    // handle unfulfilled repsonses
    fn send_with_response<T: Copy, D: QubicRequest>(&self, data: Packet<D>) -> Result<T> {
        let mut stream = TcpStream::connect(&self.url)?;

        let mut buf = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>() + std::mem::size_of::<Packet<T>>()];
        stream.write_all(data.encode_as_bytes())?;

        stream.read_exact(&mut buf)?;

        let header = unsafe { *(buf.as_ptr() as *const Header) };

        let offset = if header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers { std::mem::size_of::<Packet<ExchangePublicPeers>>() as isize } else { 0 };

        let _header = unsafe { *(buf.as_ptr().offset(offset) as *const Header) };

        let res = unsafe {
            std::ptr::read_unaligned(buf.as_ptr().offset(offset + std::mem::size_of::<Header>() as isize) as *const T)
        };

        Ok(res)
    }

    fn send_with_multiple_responses<T: Copy, D: QubicRequest>(&self, data: Packet<D>) -> Result<Vec<T>> {
        let mut ret: Vec<T> = Vec::new();

        let mut stream = TcpStream::connect(&self.url)?;

        let mut header_buffer = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>()];
        stream.write_all(data.encode_as_bytes())?;
        stream.read_exact(&mut header_buffer)?;
        header_buffer = vec![0; std::mem::size_of::<Header>()];
        let mut data_buffer = vec![0; std::mem::size_of::<T>()];

        loop {
            stream.read_exact(&mut header_buffer)?;

            let header = unsafe { *(header_buffer.as_ptr() as *const Header) };


            if header.message_type == MessageType::EndResponse {
                break;
            }

            
            stream.read_exact(&mut data_buffer)?;

            let res = unsafe {
                std::ptr::read_unaligned(data_buffer.as_ptr() as *const T)
            };

            ret.push(res);

            let size = header.get_size() - std::mem::size_of::<Header>() - std::mem::size_of::<T>();

            if size > 0 {
                stream.read_exact(&mut vec![0; size])?;
            }
        }
        
        Ok(ret)
    }

    fn connect(&self) -> Result<TcpStream> {
        Ok(TcpStream::connect(&self.url)?)
    }
}