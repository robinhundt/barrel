use std::io;
use std::io::Write;
use crate::{Error, Msg, Pos, Response, Rgba, Size};

impl Msg {
    pub(crate) fn encode<W: Write>(&self, buf: &mut W) -> Result<(), io::Error> {
        match self {
            Msg::SetPx(pos, rgb) => {
                buf.write_all(b"PX ")?;
                pos.encode(buf)?;
                buf.write_all(b" ")?;
                rgb.encode(buf)?;
            }
            Msg::GetPx(pos) => {
                buf.write_all(b"PX ")?;
                pos.encode(buf)?;
            }
            Msg::GetSize => {
                buf.write_all(b"SIZE")?;
            }
            Msg::Help => {
                buf.write_all(b"HELP")?;
            }
        }
        buf.write_all(b"\n")?;
        Ok(())
    }

    pub(crate) fn expect_response(&self) -> bool {
        match self {
            Msg::SetPx(_, _) => false,
            Msg::GetPx(_) | Msg::GetSize | Msg::Help => true,
        }
    }
}

impl Response {
    pub(crate) fn decode(buf: &str) -> Result<(&str, Self), Error> {
        match buf.as_bytes() {
            [b'P', b'X', b' ', ..] => {
                let buf = &buf[3..];
                let (buf, pos) = Pos::decode(buf)?;
                let buf = buf.trim_start_matches(char::is_whitespace);
                let (buf, rgba) = Rgba::decode(buf)?;
                Ok((buf, Self::Px(pos, rgba)))
            },
            [b'S', b'I', b'Z', b'E', b' ', ..] => {
                let buf = &buf[5..];
                let (buf, size) = Size::decode(buf)?;
                Ok((buf, Self::Size(size)))
            },
            _ => {
                Ok(("", Self::Help(buf.to_string())))
            }
        }
    }
}

impl Pos {
    #[inline]
    pub(crate) fn encode<W: Write>(&self, buf: &mut W) -> Result<(), io::Error> {
        let mut itoa_buf = itoa::Buffer::new();
        buf.write_all(itoa_buf.format(self.x).as_bytes())?;
        buf.write_all(b" ")?;
        buf.write_all(itoa_buf.format(self.y).as_bytes())?;
        Ok(())
    }

    pub(crate) fn decode(buf: &str) -> Result<(&str, Self), Error> {
        let (buf, [x, y]) = decode_two_u32(buf)?;
        let pos = Pos { x, y };
        Ok((buf, pos))

    }
}

impl Size {
    pub(crate) fn decode(buf: &str) -> Result<(&str, Self), Error> {
        let (buf, [x, y]) = decode_two_u32(buf)?;
        let size = Size { x, y };
        Ok((buf, size))
    }
}

pub(crate) fn decode_two_u32(buf: &str) -> Result<(&str, [u32; 2]), Error> {
    let mut consumed = 0;
    let mut it = buf.split_ascii_whitespace().map(|s| {
        consumed  += s.len();
        s.parse()
    });
    let x = it.next().ok_or(Error::MissingData)??;
    let y = it.next().ok_or(Error::MissingData)??;
    Ok((&buf[consumed..], [x, y]))
}

impl Rgba {
    #[inline]
    pub(crate) fn encode<W: Write>(&self, buf: &mut W) -> Result<(), io::Error> {
        buf.write_all(&fast_byte_to_hex(self.r))?;
        buf.write_all(&fast_byte_to_hex(self.g))?;
        buf.write_all(&fast_byte_to_hex(self.b))?;
        if let Some(a) = self.a {
            buf.write_all(&fast_byte_to_hex(a))?;
        }
        Ok(())
    }

    pub(crate) fn decode(buf: &str) -> Result<(&str, Self), Error> {
        if buf.len() < 6 {
            return Err(Error::MissingData);
        }
        let r = u8::from_str_radix(&buf[0..2], 16)?;
        let g = u8::from_str_radix(&buf[2..4], 16)?;
        let b = u8::from_str_radix(&buf[4..6], 16)?;
        let a = if buf.len() >= 8 {
            let a = u8::from_str_radix(&buf[6..8], 16)?;
            Some(a)
        } else {
            None
        };
        let rgba = Self {
            r,
            g,
            b,
            a,
        };
        Ok((&buf[8..], rgba))
    }
}

#[inline]
fn fast_byte_to_hex(b: u8) -> [u8; 2] {
    let nibble_to_hex = |b: u8| {
        match b {
            n @ 0..=9 => b'0' + n,
            n @ 10..=15 => b'a' + n - 10,
            _ => unreachable!()
        }
    };
    let low_nibble = nibble_to_hex(b & 0b00001111_u8);
    let high_nibble = nibble_to_hex(b >> 4);
    [high_nibble, low_nibble]
}

#[cfg(test)]
mod tests {
    use crate::{Msg, Pos, Rgba};
    use crate::codec::fast_byte_to_hex;

    #[test]
    fn pos_encode() {
        let pos = Pos::new(34, 65);
        let mut buf = vec![];
        pos.encode(&mut buf).unwrap();
        assert_eq!(&buf, "34 65".as_bytes());
    }

    #[test]
    fn rgba_encode()  {
        let col = Rgba::new(1, 11, 3, Some(4));
        let mut buf = vec![];
        col.encode(&mut buf).unwrap();
        assert_eq!(&buf, "010b0304".as_bytes())
    }

    #[test]
    fn set_px_encode() {
        let pos = Pos::new(34, 54);
        let col = Rgba::new(1, 2, 15, None);
        let msg = Msg::SetPx(pos, col);
        let mut buf = vec![];
        msg.encode(&mut buf).unwrap();
        assert_eq!(&buf, "PX 34 54 01020f\n".as_bytes());
    }

    #[test]
    fn byte_to_hex() {
        for b in 0..255_u8 {
            let hex = fast_byte_to_hex(b);
            let exp = format!("{:02x}", b);
            assert_eq!(exp, String::from_utf8(hex.to_vec()).unwrap());
        }
    }
}