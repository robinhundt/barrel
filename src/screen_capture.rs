use std::{io, mem};
use std::io::Write;
use captrs::{Bgr8, CaptureError, Capturer};
use thiserror::Error;
use crate::{Msg, Pos, Rgba};

pub struct ScreenWriter {
    capturer: Capturer,
    mode: Mode,
    previous: Vec<captrs::Bgr8>
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to create screen capturer")]
    CapturerCreate(String),
    #[error("Unable to capture screen")]
    Capture(CaptureError),
    #[error("Unable to send PX command")]
    SendPixel(#[from] io::Error)
}

pub enum Mode {
    SendAll,
    SendDiff
}

impl ScreenWriter {
    pub fn new(capture_src: usize, mode: Mode) -> Result<Self, Error> {
        let capturer = Capturer::new(capture_src).map_err(Error::CapturerCreate)?;
        Ok(Self { capturer, mode, previous: vec![] })
    }

    pub fn capture<W: Write>(&mut self, buf: &mut W) -> Result<(), Error> {
        match self.mode {
            Mode::SendAll => self.capture_all(buf),
            Mode::SendDiff => self.capture_diff(buf)
        }
    }

    fn capture_all<W:Write>(&mut self, buf: &mut W) -> Result<(), Error> {
        let (dimx, _dimy) = self.capturer.geometry();
        self.capturer.capture_store_frame().map_err(Error::Capture)?;
        let img = self.capturer.get_stored_frame().expect("Frame was stored earlier");
        for (y, row ) in  img.chunks_exact(dimx as usize).enumerate() {
            for (x, px) in row.iter().enumerate() {
                let pos = Pos::new(x as u32, y as u32);
                let col = px.into();
                Msg::SetPx(pos, col).encode(buf)?;
            }
        }
        Ok(())
    }


    fn capture_diff<W:Write>(&mut self, buf: &mut W) -> Result<(), Error> {
        let (dimx, _dimy) = self.capturer.geometry();
        let prev = match self.capturer.get_stored_frame() {
            Some(prev) => prev,
            None => {
                self.capturer.capture_frame().map_err(Error::Capture)?;
                let prev = self.capturer.get_stored_frame().expect("frame was stored");
                prev
            }
        };
        self.previous.clear();
        self.previous.extend_from_slice(prev);

        self.capturer.capture_store_frame().map_err(Error::Capture)?;
        let img = self.capturer.get_stored_frame().expect("Frame was stored earlier");
        for (y, (row, prev_row)) in  img.chunks_exact(dimx as usize).zip(self.previous.chunks_exact(dimx as usize)).enumerate() {
            for (x, (px, prev_px)) in row.iter().zip(prev_row).enumerate() {
                if px == prev_px {
                    continue;
                }
                let pos = Pos::new(x as u32, y as u32);
                let col = px.into();
                Msg::SetPx(pos, col).encode(buf)?;
            }
        }
        Ok(())
    }
}

impl From<&captrs::Bgr8> for Rgba {
    fn from(v: &captrs::Bgr8) -> Self {
        Self {
            r: v.r,
            g: v.g,
            b: v.b,
            a: None,
        }
    }
}