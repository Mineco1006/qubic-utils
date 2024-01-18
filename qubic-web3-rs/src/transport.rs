
use std::{time::Duration, convert::Infallible, cell::RefCell};
#[cfg(not(any(feature = "async", feature = "http")))]
use std::{net::TcpStream, io::{Write, Read}};

#[cfg(any(feature = "async", feature = "http"))]
use tokio::net::TcpStream;

use anyhow::Result;

use qubic_tcp_types::{Header, types::{Packet, ExchangePublicPeers}, MessageType, utils::QubicRequest};
use qubic_types::traits::{ToBytes, FromBytes};

#[cfg(any(feature = "async", feature = "http"))]
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[cfg(not(any(feature = "async", feature = "http")))]
pub trait Transport {
    type Err;

    fn new(url: String) -> Result<Box<Self>, Self::Err>;

    fn send_without_response(&self, data: impl ToBytes) -> Result<()>;

    fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T>;

    fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>>;

    fn get_url(&self) -> String;
 
    fn connect(&self) -> Result<TcpStream>;
}

#[cfg(any(feature = "async", feature = "http"))]
#[async_trait::async_trait]
pub trait Transport {
    async fn new(url: String) -> Self;

    async fn send_without_response<B: ToBytes + Send>(&self, data: B) -> Result<()>;

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

    async fn send_without_response<B: ToBytes + Send>(&self, data: B) -> Result<()> {
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
    type Err = Infallible;

    fn new(url: String) -> Result<Box<Self>, Self::Err> {
        Ok(Box::new(Self {
            url
        }))
    }

    fn send_without_response(&self, data: impl ToBytes) -> Result<()> {
        let mut stream = TcpStream::connect(&self.url)?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        stream.write_all(&data.to_bytes())?;

        Ok(())
    }

    fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T> {
        let mut stream = TcpStream::connect(&self.url)?;

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let mut header_buffer = vec![0; std::mem::size_of::<Header>()];
        stream.write_all(&data.to_bytes())?;

        stream.read_exact(&mut header_buffer)?;

        let mut header = Header::from_bytes(&header_buffer)?;

        let offset = header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers;

        if offset {
            let mut flush_buf = vec![0; header.get_size() - std::mem::size_of::<Header>()];

            stream.read_exact(&mut flush_buf)?;
            drop(flush_buf);

            stream.read_exact(&mut header_buffer)?;

            header = Header::from_bytes(&header_buffer)?;
        }

        let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

        stream.read_exact(&mut data_buffer)?;

        let res = T::from_bytes(&data_buffer)?;

        Ok(res)
    }

    fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>> {
        let mut ret: Vec<T> = Vec::new();

        let mut stream = TcpStream::connect(&self.url)?;

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let mut header_buffer = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>()];
        stream.write_all(&data.to_bytes())?;
        stream.read_exact(&mut header_buffer)?;
        header_buffer = vec![0; std::mem::size_of::<Header>()];

        loop {
            stream.read_exact(&mut header_buffer)?;

            let header = unsafe { *(header_buffer.as_ptr() as *const Header) };


            if header.message_type == MessageType::EndResponse {
                break;
            }

            let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

            
            stream.read_exact(&mut data_buffer)?;

            let res = T::from_bytes(&data_buffer)?;

            ret.push(res);
        }
        
        Ok(ret)
    }

    fn get_url(&self) -> String {
        self.url.clone()
    }

    fn connect(&self) -> Result<TcpStream> {
        Ok(TcpStream::connect(&self.url)?)
    }
}

pub struct ConnectedTcp {
    pub stream: RefCell<TcpStream>
}

#[cfg(not(any(feature = "async", feature = "http")))]
impl Transport for ConnectedTcp {
    type Err = std::io::Error;

    fn new(url: String) -> Result<Box<Self>, Self::Err> {
        let stream = TcpStream::connect(&url)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(
            Box::new(Self {
                stream: RefCell::new(stream)
            })
        )
    }

    fn send_without_response(&self, data: impl ToBytes) -> Result<()> {
        match self.stream.borrow_mut().write_all(&data.to_bytes()) {

            // auto reconnection
            Err(e) => {
                *self.stream.borrow_mut() = TcpStream::connect(self.get_url())?;

                return Err(e.into())
            },
            _ => ()
        };

        Ok(())
    }

    fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T> {

        let res: Result<T> = {
            self.stream.borrow_mut().flush()?;

            let mut header_buffer = vec![0; std::mem::size_of::<Header>()];
            self.stream.borrow_mut().write_all(&data.to_bytes())?;

            self.stream.borrow_mut().read_exact(&mut header_buffer)?;

            let mut header = Header::from_bytes(&header_buffer)?;

            let offset = header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers;

            if offset {
                let mut flush_buf = vec![0; header.get_size() - std::mem::size_of::<Header>()];

                self.stream.borrow_mut().read_exact(&mut flush_buf)?;
                drop(flush_buf);

                self.stream.borrow_mut().read_exact(&mut header_buffer)?;

                header = Header::from_bytes(&header_buffer)?;
            }

            let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

            self.stream.borrow_mut().read_exact(&mut data_buffer)?;

            let res = T::from_bytes(&data_buffer)?;

            Ok(res)
        };
        

        match res {
            Ok(r) => Ok(r),
            Err(e) => {
                *self.stream.borrow_mut() = TcpStream::connect(self.get_url())?;

                Err(e.into())
            }
        }
    }

    fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>> {

        let res: Result<Vec<T>> = {
            let mut ret: Vec<T> = Vec::new();
            self.stream.borrow_mut().flush()?;
            self.stream.borrow_mut().write_all(&data.to_bytes())?;
            let mut header_buffer = vec![0; std::mem::size_of::<Header>()];

            loop {
                self.stream.borrow_mut().read_exact(&mut header_buffer)?;

                let header = unsafe { *(header_buffer.as_ptr() as *const Header) };


                if header.message_type == MessageType::EndResponse {
                    break;
                }

                let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

                
                self.stream.borrow_mut().read_exact(&mut data_buffer)?;

                let res = T::from_bytes(&data_buffer)?;

                ret.push(res);
            }
            
            Ok(ret)
        };
        
        match res {
            Ok(r) => Ok(r),
            Err(e) => {
                *self.stream.borrow_mut() = TcpStream::connect(self.get_url())?;

                Err(e.into())
            }
        }
    }

    fn get_url(&self) -> String {
        self.stream.borrow().peer_addr().unwrap().to_string()
    }

    fn connect(&self) -> Result<TcpStream> {
        Ok(self.stream.borrow().try_clone()?)
    }
}