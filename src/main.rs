use std::mem;
use barrel::{Client, Msg, Pos, Rgba};
use barrel::camera::CameraWriter;
use barrel::image_writer::GifWriter;
use barrel::screen_capture::{Mode, ScreenWriter};
// use barrel::image_writer::GifWriter;

fn main() {

    let mut client = Client::connect("192.168.120.119:1234").unwrap();

    // let mut cam_writer = CameraWriter::new(0).unwrap();
    //
    // loop {
    //     client.send_camera_capture(&mut cam_writer).unwrap()
    // }

    let mut screen_writer = ScreenWriter::new(0, Mode::SendAll).unwrap();
    loop {
        client.send_capture(&mut screen_writer).unwrap()
    }

    // let mut msgs = vec![];
    // for x in 0..600 {
    //     for y in 0..600 {
    //         let pos = Pos::new(x, y);
    //         let col = Rgba::blue();
    //         msgs.push(Msg::SetPx(pos, col));
    //     }
    // }

    let gif_writer = GifWriter::load("images/rick-roll.gif").unwrap();

    loop {
        client.send_gif(&gif_writer).unwrap();
    }
    // dbg!(client.send(Msg::SetPx(Pos {})));
}

