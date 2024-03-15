use std::fs::File;
use std::io;
use std::io::{BufReader, Write};
use std::path::Path;
use std::time::Duration;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, ImageError};
use thiserror::Error;
use crate::{Msg, Pos, Rgba};

pub struct GifWriter {
    msg_buf: Vec<Frame>
}

#[derive(Clone, Debug)]
pub(crate) struct Frame {
    msgs: Vec<Msg>,
    delay: Duration
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to open file")]
    Open(#[from] io::Error),
    #[error("Unable to decode image")]
    Decode(#[from] ImageError)
}

impl GifWriter {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let reader = BufReader::new(File::open(path)?);
        let decoder = GifDecoder::new(reader)?;
        let msg_buf = msg_frames(decoder)?;
        Ok(Self {
            msg_buf,
        })
    }

    pub(crate) fn frames(&self) -> &[Frame] {
        &self.msg_buf
    }
}

impl Frame {
    pub(crate) fn encode<W: Write>(&self, buf: &mut W) -> Result<(), io::Error> {
        for msg in &self.msgs {
            msg.encode(buf)?;
        }
        Ok(())
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }
}

fn msg_frames(decoder: GifDecoder<BufReader<File>>) -> Result<Vec<Frame>, ImageError> {
    let mut msg_frames = vec![];
    for frame in decoder.into_frames() {
        let frame = frame?;
        let mut msgs = Vec::with_capacity(frame.buffer().len());
        for (y, row) in frame.buffer().rows().enumerate() {
            for (x, px) in row.enumerate() {
                let pos = Pos::new(x as u32, y as u32);
                let col = Rgba::from(px);
                msgs.push(Msg::SetPx(pos, col));
            }
        }
        let (num, denum) = frame.delay().numer_denom_ms();
        let delay = Duration::from_secs_f64(num as f64 / (denum as f64 * 1000.0));
        msg_frames.push(Frame { msgs, delay });
    }
    Ok(msg_frames)
}

impl From<&image::Rgba<u8>> for Rgba {
    fn from(v: &image::Rgba<u8>) -> Self {
        Self {
            r: v.0[0],
            g: v.0[1],
            b: v.0[2],
            a: Some(v.0[3]),
        }
    }
}
