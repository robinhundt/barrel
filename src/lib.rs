use std::io::{self, BufRead, BufReader, BufWriter, Lines, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::num::ParseIntError;
use thiserror::Error;

mod codec;
#[cfg(feature = "image")]
pub mod image_writer;
#[cfg(feature = "camera")]
pub mod camera;

#[cfg(feature = "capture")]
pub mod screen_capture;

pub struct Client {
    write: BufWriter<TcpStream>,
    read: Lines<BufReader<TcpStream>>,
}

impl Client {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr).map_err(Error::Connect)?;
        let write = BufWriter::new(stream.try_clone().unwrap());
        let read = BufReader::new(stream).lines();
        Ok(Self { write, read })
    }


    /// Flushes the internal buffer after sending.
    pub fn send(&mut self, msg: Msg) -> Result<(), Error> {
        self.send_buffered(msg)?;
        self.flush()?;
        Ok(())
    }

    /// Send and receive a response. For message which don't
    /// [`Msg::expect_response`], an error is returned.
    pub fn send_recv(&mut self, msg: Msg) -> Result<Response, Error> {
        if !msg.expect_response() {
            return Err(Error::NoResponseExpected);
        }
        self.send(msg)?;
        let resp = self.recv()?;
        Ok(resp)
    }

    /// Flushes after sending all messages.
    pub fn send_all(&mut self, msgs: &[Msg]) -> Result<Vec<Response>, Error> {
        let mut expected_responses = 0;
        for msg in msgs {
            if msg.expect_response() {
                expected_responses += 1;
            }
            self.send_buffered(*msg)?;
        }
        self.flush()?;
        let mut responses = Vec::with_capacity(expected_responses);
        while responses.len() < expected_responses {
            responses.push(self.recv()?);
        }
        Ok(responses)
    }

    /// Does not explicitly flush the buffer after sending.
    /// Use [`Client::send`] or manual [`Client::flush`].
    #[inline]
    pub fn send_buffered(&mut self, msg: Msg) -> Result<(), Error> {
        msg.encode(&mut self.write).map_err(Error::SendCmd)?;
        Ok(())
    }

    /// Send a gif loaded via the [`GifWriter`] API. Needs **features = ["image"]**.
    #[cfg(feature = "image")]
    pub fn send_gif(&mut self, gif: &image_writer::GifWriter) -> Result<(), Error> {
        for frame in gif.frames() {
            frame.encode(&mut self.write).map_err(Error::SendCmd)?;
            self.flush()?;
            std::thread::sleep(frame.delay());
        }
        Ok(())
    }

    #[cfg(feature = "capture")]
    pub fn send_capture(&mut self, screen_writer: &mut screen_capture::ScreenWriter) -> Result<(), Error> {
        screen_writer.capture(&mut self.write)?;
        self.flush()?;
        Ok(())
    }

    #[cfg(feature = "camera")]
    pub fn send_camera_capture(&mut self, camera_writer: &mut camera::CameraWriter) -> Result<(), Error> {
        camera_writer.capture(&mut self.write)?;
        self.flush()?;
        Ok(())
    }

    #[inline]
    fn recv(&mut self) -> Result<Response, Error> {
        let line = self.read.next().ok_or(Error::MissingData)?.map_err(Error::Receive)?;
        let line = line.as_str();
        let (_rest, resp) = Response::decode(line)?;
        Ok(resp)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.write.flush().map_err(Error::SendCmd)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to connect")]
    Connect(#[source] io::Error),
    #[error("Unable to send command")]
    SendCmd(#[source] io::Error),
    #[error("Receiver failed")]
    Receive(#[source] io::Error),
    #[error("Missing data in response")]
    MissingData,
    #[error("Unable to decode rgba color")]
    RgbaDecode(#[from] ParseIntError),
    #[error("The msg to sent expects no response. Use send.")]
    NoResponseExpected,
    #[cfg(feature = "capture")]
    #[error("Unable to send screen capture")]
    ScreenCapture(#[from] screen_capture::Error),
    #[cfg(feature = "camera")]
    #[error("Unable to send camera capture")]
    CameraCapture(#[from] camera::Error)
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    SetPx(Pos, Rgba),
    GetPx(Pos),
    GetSize,
    Help
}

#[derive(Debug, Clone)]
pub enum Response {
    Px(Pos, Rgba),
    Size(Size),
    Help(String)
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub x: u32,
    pub y: u32
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Pos {
    pub x: u32,
    pub y: u32
}

impl Pos {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
        }
    }

}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: Option<u8>
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: Option<u8>) -> Self {
        Self {
            r,
            g,
            b,
            a,
        }
    }

    pub fn black() -> Self {
        Self::new(255, 255, 255, None)
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0, None)
    }

    pub fn green() -> Self {
        Self::new(0, 255, 0, None)
    }

    pub fn blue() -> Self {
        Self::new(0, 0, 255, None)
    }
}