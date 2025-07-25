
use std::{cell::RefCell, convert::Infallible, time::Duration};
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

    fn new(url: String, timeout: Option<std::time::Duration>) -> Result<Box<Self>, Self::Err>;

    fn send_without_response<D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<()>;

    fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T>;

    fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>>;

    fn get_url(&self) -> String;
 
    fn connect(&self) -> Result<TcpStream>;
}

#[cfg(any(feature = "async", feature = "http"))]
pub trait Transport {
    type Err;

    async fn new(url: String, timeout: Option<std::time::Duration>) -> Result<Box<Self>, Self::Err>;

    async fn send_without_response(&self, data: impl ToBytes) -> Result<()>;

    async fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T>;

    async fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>>;

    async fn get_url(&self) -> String;
 
    async fn connect(&self) -> Result<TcpStream>;
}

pub struct Tcp {
    pub(crate) url: String,
    pub(crate) timeout: Duration
}

/// Default timeout: 5s
#[cfg(any(feature = "async", feature = "http"))]
impl Transport for Tcp {
    type Err = Infallible;

    async fn new(url: String, timeout: Option<Duration>) -> Result<Box<Self>, Self::Err> {
        Ok(Box::new(Self {
            url,
            timeout: if let Some(timeout) = timeout { timeout } else { std::time::Duration::from_secs(5) }
        }))
    }

    async fn send_without_response(&self, data: impl ToBytes) -> Result<()> {
        let std_stream = std::net::TcpStream::connect(&self.url)?;

        std_stream.set_read_timeout(Some(self.timeout))?;
        std_stream.set_write_timeout(Some(self.timeout))?;

        let mut stream = TcpStream::from_std(std_stream)?;

        stream.write_all(&data.to_bytes()).await?;

        Ok(())
    }

    async fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T> {
        let std_stream = std::net::TcpStream::connect(&self.url)?;

        std_stream.set_read_timeout(Some(self.timeout))?;
        std_stream.set_write_timeout(Some(self.timeout))?;

        let mut stream = TcpStream::from_std(std_stream)?;

        let mut header_buffer = vec![0; std::mem::size_of::<Header>()];
        stream.write_all(&data.to_bytes()).await?;

        stream.read_exact(&mut header_buffer).await?;

        let mut header = Header::from_bytes(&header_buffer).unwrap();

        let offset = header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers;

        if offset {
            let mut flush_buf = vec![0; header.get_size() - std::mem::size_of::<Header>()];

            stream.read_exact(&mut flush_buf).await?;
            drop(flush_buf);

            stream.read_exact(&mut header_buffer).await?;

            header = Header::from_bytes(&header_buffer).unwrap();
        }

        let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

        stream.read_exact(&mut data_buffer).await?;

        let res = T::from_bytes(&data_buffer).unwrap();

        Ok(res)
    }

    async fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>> {
        let mut ret: Vec<T> = Vec::new();

        let std_stream = std::net::TcpStream::connect(&self.url)?;

        std_stream.set_read_timeout(Some(self.timeout))?;
        std_stream.set_write_timeout(Some(self.timeout))?;

        let mut stream = TcpStream::from_std(std_stream)?;

        let mut header_buffer = vec![0; std::mem::size_of::<Packet<ExchangePublicPeers>>()];
        stream.write_all(&data.to_bytes()).await?;
        stream.read_exact(&mut header_buffer).await?;
        header_buffer = vec![0; std::mem::size_of::<Header>()];

        loop {
            stream.read_exact(&mut header_buffer).await?;

            let header = Header::from_bytes(&header_buffer).unwrap();

            if header.message_type == MessageType::EndResponse {
                break;
            }

            let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

            
            stream.read_exact(&mut data_buffer).await?;

            let res = T::from_bytes(&data_buffer).unwrap();

            ret.push(res);
        }
        
        Ok(ret)
    }

    async fn get_url(&self) -> String {
        self.url.clone()
    }

    async fn connect(&self) -> Result<TcpStream> {
        Ok(TcpStream::connect(&self.url).await?)
    }
}

#[cfg(not(any(feature = "async", feature = "http")))]
impl Transport for Tcp {
    type Err = Infallible;

    /// defaults timeout to 5 seconds if parameter is None
    fn new(url: String, timeout: Option<Duration>) -> Result<Box<Self>, Self::Err> {
        Ok(Box::new(Self {
            url,
            timeout: if let Some(timeout) = timeout { timeout } else { std::time::Duration::from_secs(5) }
        }))
    }

    fn send_without_response<D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<()> {
        let mut stream = TcpStream::connect(&self.url)?;
        stream.set_write_timeout(Some(self.timeout))?;

        stream.write_all(&data.to_bytes())?;
        Ok(())
    }

    fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T> {
        let mut stream = TcpStream::connect(&self.url)?;

        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

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

        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

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
    pub stream: RefCell<TcpStream>,
    pub url: String,
    timeout: Duration
}

#[cfg(not(any(feature = "async", feature = "http")))]
impl Transport for ConnectedTcp {
    type Err = std::io::Error;

    fn new(url: String, timeout: Option<std::time::Duration>) -> Result<Box<Self>, Self::Err> {
        let stream = TcpStream::connect(&url)?;
        let timeout = if let Some(timeout) = timeout { timeout } else { Duration::from_secs(5) };
 
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(
            Box::new(Self {
                stream: RefCell::new(stream),
                url,
                timeout
            })
        )
    }

    fn send_without_response<D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<()> {
        let mut self_stream = self.stream.borrow_mut();
        match self_stream.write_all(&data.to_bytes()) {

            // auto reconnection
            Err(e) => {
                let addr = self.get_url();
                let stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(self.timeout))?;
                stream.set_write_timeout(Some(self.timeout))?;
                *self_stream = stream;

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
                let stream = TcpStream::connect(self.get_url())?;
                stream.set_read_timeout(Some(self.timeout))?;
                stream.set_write_timeout(Some(self.timeout))?;
                *self.stream.borrow_mut() = stream;

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
                let stream = TcpStream::connect(self.get_url())?;
                stream.set_read_timeout(Some(self.timeout))?;
                stream.set_write_timeout(Some(self.timeout))?;
                *self.stream.borrow_mut() = stream;

                Err(e.into())
            }
        }
    }

    fn get_url(&self) -> String {
        self.url.clone()
    }

    fn connect(&self) -> Result<TcpStream> {
        Ok(self.stream.borrow().try_clone()?)
    }
}

#[cfg(any(feature = "async", feature = "http"))]
impl Transport for ConnectedTcp {
    type Err = std::io::Error;

    async fn new(url: String, timeout: Option<std::time::Duration>) -> Result<Box<Self>, Self::Err> {
        let std_stream = std::net::TcpStream::connect(url.clone())?;
        let timeout = if let Some(timeout) = timeout { timeout } else { Duration::from_secs(5) };

        std_stream.set_read_timeout(Some(timeout))?;
        std_stream.set_write_timeout(Some(timeout))?;
        let stream = TcpStream::from_std(std_stream)?;
        Ok(
            Box::new(Self {
                stream: RefCell::new(stream),
                url,
                timeout
            })
        )
    }

    async fn send_without_response(&self, data: impl ToBytes) -> Result<()> {
        if let Err(e) = self.stream.borrow_mut().write_all(&data.to_bytes()).await {
            let std_stream = std::net::TcpStream::connect(self.get_url().await)?;
            std_stream.set_read_timeout(Some(self.timeout))?;
            std_stream.set_write_timeout(Some(self.timeout))?;
            *self.stream.borrow_mut() = TcpStream::from_std(std_stream)?;

            return Err(e.into())
        }

        Ok(())
    }

    async fn send_with_response<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<T> {

        let res: Result<T> = {

            self.stream.borrow_mut().flush().await?;

            let mut header_buffer = vec![0; std::mem::size_of::<Header>()];
            self.stream.borrow_mut().write_all(&data.to_bytes()).await?;

            self.stream.borrow_mut().read_exact(&mut header_buffer).await?;

            let mut header = Header::from_bytes(&header_buffer).unwrap();

            let offset = header.message_type == MessageType::ExchangePublicPeers && D::get_message_type() != MessageType::ExchangePublicPeers;

            if offset {
                let mut flush_buf = vec![0; header.get_size() - std::mem::size_of::<Header>()];

                self.stream.borrow_mut().read_exact(&mut flush_buf).await?;
                drop(flush_buf);

                self.stream.borrow_mut().read_exact(&mut header_buffer).await?;

                header = Header::from_bytes(&header_buffer).unwrap();
            }

            let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];

            self.stream.borrow_mut().read_exact(&mut data_buffer).await?;

            let res = T::from_bytes(&data_buffer).unwrap();

            Ok(res)
        };
        

        match res {
            Ok(r) => Ok(r),
            Err(e) => {
                let std_stream = std::net::TcpStream::connect(self.get_url().await)?;
                std_stream.set_read_timeout(Some(self.timeout))?;
                std_stream.set_write_timeout(Some(self.timeout))?;
                *self.stream.borrow_mut() = TcpStream::from_std(std_stream)?;

                Err(e.into())
            }
        }
    }

    async fn send_with_multiple_responses<T: FromBytes, D: QubicRequest + ToBytes>(&self, data: Packet<D>) -> Result<Vec<T>> {

        let res: Result<Vec<T>> = {
            let mut ret: Vec<T> = Vec::new();
            let mut stream = self.stream.borrow_mut();

            stream.flush().await?;
            stream.write_all(&data.to_bytes()).await?;
            let mut header_buffer = vec![0; std::mem::size_of::<Header>()];

            loop {
                stream.read_exact(&mut header_buffer).await?;
    
                let header = Header::from_bytes(&header_buffer).unwrap();
    
                if header.message_type == MessageType::EndResponse {
                    break;
                }
    
                let mut data_buffer = vec![0; header.get_size() - std::mem::size_of::<Header>()];
    
                
                stream.read_exact(&mut data_buffer).await?;
    
                let res = T::from_bytes(&data_buffer).unwrap();
    
                ret.push(res);
            }
            
            Ok(ret)
        };
        
        match res {
            Ok(r) => Ok(r),
            Err(e) => {
                let std_stream = std::net::TcpStream::connect(self.get_url().await)?;
                std_stream.set_read_timeout(Some(self.timeout))?;
                std_stream.set_write_timeout(Some(self.timeout))?;
                *self.stream.borrow_mut() = TcpStream::from_std(std_stream)?;

                Err(e)
            }
        }
    }

    async fn get_url(&self) -> String {
        self.stream.borrow().peer_addr().unwrap().to_string()
    }

    async fn connect(&self) -> Result<TcpStream> {
        Ok(TcpStream::connect(self.get_url().await).await?)
    }
}