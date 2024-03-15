use std::io;
use std::io::Write;
use thiserror::Error;
use v4l::buffer::Type;
use v4l::{Device, FourCC};
use v4l::format::Colorspace;
use v4l::io::traits::{CaptureStream, Stream};
use v4l::prelude::UserptrStream;
use v4l::video::Capture;
use crate::{Msg, Pos, Rgba};

// TODO maybe use v4l directly as nokhwa seems to not build

pub struct CameraWriter {
    dev: Device,
    stream: UserptrStream,
    dim: (u32, u32)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to send camera frame")]
    SendCameraFrame(#[from] io::Error),
    // #[error("Unable to get camera frame")]
    // GetCameraFrame(#[source] NokhwaError),
    // #[error("Unable to set up camera")]
    // CameraSetup(#[source] NokhwaError)
}

impl CameraWriter {
    pub fn new(camera_id: usize) -> Result<Self, Error> {
        let dev = Device::new(camera_id).unwrap();
        let mut fmt = dev.format().unwrap();
        dbg!(&fmt);
        fmt.fourcc = FourCC::new(b"RGB3");
        let mut stream = UserptrStream::new(&dev, Type::VideoCapture).unwrap();
        stream.start().unwrap();
        // camera.set_frame_format(FrameFormat::RAWRGB).map_err(Error::CameraSetup)?;
        // camera.open_stream().map_err(Error::CameraSetup)?;
        Ok(Self { dev, stream, dim: (fmt.width, fmt.height) })
    }

    pub fn capture<W: Write>(&mut self, buf: &mut W) -> Result<(), Error> {
        let (frame, _meta) = self.stream.next().unwrap();
        println!("{:?}", frame);

        // four three for rgb pixel size
        for (y, row ) in  frame.chunks_exact(self.dim.0 as usize * 3).enumerate() {
            for (x, px) in row.chunks_exact(3).enumerate() {
                let pos = Pos::new(x as u32, y as u32);
                let &[r, g, b] = px else {
                    unreachable!("chunk_exact by 3");
                };
                let col = Rgba::new(r, g, b, None);
                Msg::SetPx(pos, col).encode(buf)?;
            }
        }

        Ok(())
    }
}