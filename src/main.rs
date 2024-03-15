use std::collections::btree_map::Entry;
use std::{fs, mem};
use barrel::{Client, Msg, Pos, Rgba};
use barrel::camera::CameraWriter;
use barrel::image_writer::GifWriter;
use barrel::screen_capture::{Mode, ScreenWriter};
// use barrel::image_writer::GifWriter;

fn main() {

    let mut client = Client::connect("192.168.120.119:1234").unwrap();

    // let size = client.get_size().unwrap();
    // let mut gif_writer = GifWriter::load("images/bad-apple.gif").unwrap();
    // loop {
    //     client.send_gif(&mut gif_writer).unwrap();
    // }

    // loop {
    //     for x in 0 .. size.x {
    //         for y in 0..size.y {
    //             if (x < 50 || x < size.x / 2 + 50) && y < 50 {
    //                 continue;
    //             }
    //             let pos = Pos::new(x, y);
    //             let col = Rgba::new(229, 65, 251, None);
    //             client.send_buffered(Msg::SetPx(pos, col)).unwrap();
    //         }
    //     }
    //     client.flush().unwrap();
    // }

    let mut cam_writer = CameraWriter::new(0).unwrap();
    
    loop {
        client.send_camera_capture(&mut cam_writer).unwrap()
    } 

    // let mut screen_writer = ScreenWriter::new(0, Mode::SendDiff).unwrap();
    // loop {
    //     client.send_capture(&mut screen_writer).unwrap()
    // }

    // let mut msgs = vec![];
    // for x in 0..600 {
    //     for y in 0..600 {
    //         let pos = Pos::new(x, y);
    //         let col = Rgba::blue();
    //         msgs.push(Msg::SetPx(pos, col));
    //     }
    // }

    // loop {
    //     for entry in fs::read_dir("images").unwrap() {
    //         let gif_writer = GifWriter::load(entry.unwrap().path()).unwrap();
    //         client.send_gif(&gif_writer).unwrap();
    //     }
    // }
}

