use std::io;
use std::io::Write;
use thiserror::Error;
use v4l::buffer::Type;
use v4l::{Device, FourCC};
use v4l::format::Colorspace;
use v4l::io::traits::{CaptureStream, Stream};
use v4l::prelude::UserptrStream;
use v4l::video::Capture;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

use crate::{Msg, Pos, Rgba};

// TODO maybe use v4l directly as nokhwa seems to not build

pub struct CameraWriter {
    dev: Device,
    stream: UserptrStream,
    dim: (u32, u32),
    current_mod: u8
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
        fmt.width = 1280;
        fmt.height = 720;
        fmt.fourcc = FourCC::new(b"MJPG");
        dev.set_format(&fmt).unwrap();
        let mut stream = UserptrStream::new(&dev, Type::VideoCapture).unwrap();
        stream.start().unwrap();
        // camera.set_frame_format(FrameFormat::RAWRGB).map_err(Error::CameraSetup)?;
        // camera.open_stream().map_err(Error::CameraSetup)?;
        Ok(Self { dev, stream, dim: (fmt.width, fmt.height) , current_mod: 1})
    }

    pub fn capture<W: Write>(&mut self, buf: &mut W) -> Result<(), Error> {
        let (frame, _meta) = self.stream.next().unwrap();
        let mut options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
        let mut decoder = JpegDecoder::new_with_options(frame,options);
        let pixels = decoder.decode().unwrap();
        // four three for rgb pixel size
        for (y, row ) in  pixels.chunks_exact(self.dim.0 as usize * 4).enumerate() {
            for (x, px) in row.chunks_exact(4).enumerate() {
                let pos = Pos::new(x as u32, y as u32);

                let &[r, g, b, a] = px else {
                    unreachable!("chunk_exact by 4");
                };
                // let mut rgb = [r, g, b];
                // rgb.rotate_left((self.current_mod % 3) as usize);
                // let [r, g, b] = rgb;
                let [r,g,b] = match self.current_mod % 3 {
                    0 => [r, 0, 0],
                    1 => [0, g, 0],
                    2 => [0, 0, b],
                    _ => unreachable!()
                };
                // let col = Rgba::new(g/ self.current_mod * self.current_mod, b/ self.current_mod * self.current_mod, r / self.current_mod * self.current_mod, Some(a));
                let col = Rgba::new(r, g, b , Some(a));
                Msg::SetPx(pos, col).encode(buf)?;
            }
        }

        self.current_mod = self.current_mod.wrapping_add(1);
        if self.current_mod == 0 {
            self.current_mod += 1;
        }

        Ok(())
    }
}