use std::path::Path;

use ffmpeg_next as ffmpeg;
use image::RgbImage;

pub fn extract_frame(path: &Path, pos: f64) -> Result<RgbImage, ffmpeg::Error> {
    ffmpeg::init()?;
    ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Quiet);

    let mut ictx = ffmpeg::format::input(&path)?;
    let input = ictx.streams();

    let stream = input
        .best(ffmpeg::media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let stream_index = stream.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let duration = ictx.duration();
    if duration <= 0 {
        return Err(ffmpeg::Error::Other { errno: 0 });
    }
    let timestamp = (duration as f64 * pos).floor() as i64;

    ictx.seek(timestamp, ..)?;

    let mut decoded = ffmpeg::util::frame::video::Video::empty();

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }

        decoder.send_packet(&packet)?;
        if decoder.receive_frame(&mut decoded).is_err() {
            continue;
        }

        let (src_width, src_height) = (decoded.width(), decoded.height());
        let dst_width = 480;
        let dst_height = (src_height as f64 / src_width as f64 * dst_width as f64) as u32;

        let mut scaler = ffmpeg::software::scaling::context::Context::get(
            decoded.format(),
            src_width,
            src_height,
            ffmpeg::format::Pixel::RGB24,
            dst_width,
            dst_height,
            ffmpeg::software::scaling::flag::Flags::BILINEAR,
        )?;

        let mut rgb_frame = ffmpeg::util::frame::video::Video::empty();
        scaler.run(&decoded, &mut rgb_frame)?;

        // Extract RGB data
        let width = rgb_frame.width();
        let height = rgb_frame.height();
        let data = rgb_frame.data(0);
        let stride = rgb_frame.stride(0);

        // Create an ImageBuffer
        let mut img = RgbImage::new(width, height);
        for (y, row) in img.rows_mut().enumerate() {
            let offset = y * stride;
            for (x, pixel) in row.enumerate() {
                let i = offset + x * 3;
                *pixel = image::Rgb([data[i], data[i + 1], data[i + 2]]);
            }
        }

        return Ok(img);
    }

    Err(ffmpeg::Error::Other { errno: 0 })
}
