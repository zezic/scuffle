use std::{ptr, time::Instant};

use rusty_ffmpeg::ffi::{
    AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX, AVCodec, AVCodecHWConfig, av_find_best_stream, av_hwdevice_find_type_by_name,
    avcodec_get_hw_config,
};
use scuffle_ffmpeg::{
    AVHWDeviceType, AVMediaType, AVPixelFormat,
    codec::DecoderCodec,
    decoder::{Decoder, DecoderOptions, HardwareAccelerationOptions},
    error::FfmpegError,
    io::Input,
};

fn main() -> Result<(), FfmpegError> {
    let hw_device_type_name = std::env::args().nth(1).expect("No HW Device Type provided");
    let filename = std::env::args().nth(2).expect("No filename provided");
    let use_hw_device = std::env::args().nth(3).expect("No use_hw_device provided");
    let use_hw_device: bool = use_hw_device.parse().expect("Invalid use_hw_device value");

    let mut input = Input::open(&filename).unwrap();

    let av_format_ctx = input.as_mut_ptr();

    let mut av_codec_ptr = ptr::null();
    let mut_ptr_to_ptr = &mut av_codec_ptr as *mut *const AVCodec;
    let _stream = unsafe { av_find_best_stream(av_format_ctx, AVMediaType::Video.into(), -1, -1, mut_ptr_to_ptr, 0) };
    let decoder_codec = unsafe { DecoderCodec::from_ptr(av_codec_ptr) };

    let c_name = std::ffi::CString::new(hw_device_type_name).ok().unwrap();
    let hw_device_type = unsafe { av_hwdevice_find_type_by_name(c_name.as_ptr()) };
    let requested_hw_device_type = AVHWDeviceType::from(hw_device_type);

    println!("Requested HW Device Type: {:?}", requested_hw_device_type);

    let codec = decoder_codec.as_ptr();

    let mut hw_pix_fmt = None;

    for i in 0.. {
        println!("i: {}", i);
        let config = unsafe { avcodec_get_hw_config(codec, i) };
        if config.is_null() {
            println!("No hw configs available");
            return Ok(());
        }
        let config: &AVCodecHWConfig = unsafe { config.as_ref().unwrap() };
        let pix_fmt = AVPixelFormat::from(config.pix_fmt);
        let device_type = AVHWDeviceType::from(config.device_type);
        println!("Pix fmt: {:#?}", pix_fmt);
        println!("Device Type: {:?}", device_type);
        let supports_hw_device_ctx = (config.methods & (AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX as i32)) != 0;
        println!("Supports HW Device Context: {}", supports_hw_device_ctx);
        if supports_hw_device_ctx && device_type == requested_hw_device_type {
            hw_pix_fmt = Some(pix_fmt);
            break;
        }
    }

    let streams = input.streams();
    let best_video_stream = streams.best(AVMediaType::Video).unwrap();

    let options = DecoderOptions {
        codec: Some(decoder_codec),
        thread_count: 1,
        hardware_acceleration: use_hw_device.then_some(HardwareAccelerationOptions {
            hw_device_type,
            hw_pixel_format: hw_pix_fmt.unwrap(),
        }),
    };
    let mut video_decoder = Decoder::with_options(&best_video_stream, options).unwrap().video().unwrap();

    let video_stream_index = best_video_stream.index();

    let mut last_frame_at = Instant::now();

    for packet in input.packets() {
        let packet = packet?;
        if packet.stream_index() == video_stream_index {
            video_decoder.send_packet(&packet)?;
            while let Some(_frame) = video_decoder.receive_frame()? {
                let elapsed = last_frame_at.elapsed();
                dbg!(&elapsed);
                last_frame_at = Instant::now();
            }
        }
    }

    video_decoder.send_eof()?;

    Ok(())
}
