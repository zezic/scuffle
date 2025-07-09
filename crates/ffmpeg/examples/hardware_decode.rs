//! Hardware-accelerated video decoding example.
//!
//! This example demonstrates how to use CUDA hardware acceleration for video decoding.
//! It shows how to:
//! - Create a hardware-accelerated decoder
//! - Handle hardware frames with automatic transfer to system memory
//! - Use different hardware acceleration backends (CUDA, VA-API)
//!
//! Usage:
//! ```bash
//! cargo run --example hardware_decode -- <input_file> [device_type]
//! ```
//!
//! Where device_type can be:
//! - cuda (default)
//! - vaapi
//! - none (software decoding)

use std::env;
use std::path::Path;

use scuffle_ffmpeg::decoder::{Decoder, DecoderOptions};
use scuffle_ffmpeg::hardware::{HardwareConfig, HardwareContext};
use scuffle_ffmpeg::io::Input;
use scuffle_ffmpeg::{AVHWDeviceType, AVMediaType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file> [device_type]", args[0]);
        eprintln!("device_type can be: cuda, vaapi, none");
        return Ok(());
    }

    let input_path = &args[1];
    let device_type = args.get(2).map(|s| s.as_str()).unwrap_or("cuda");

    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        return Ok(());
    }

    // Parse device type
    let hardware_config = match device_type {
        "cuda" => HardwareConfig::cuda(),
        "vaapi" => HardwareConfig::vaapi(),
        "none" => HardwareConfig::default(),
        _ => {
            eprintln!("Error: Unknown device type '{}'. Use: cuda, vaapi, or none", device_type);
            return Ok(());
        }
    };

    println!("Using hardware acceleration: {}", hardware_config.device_type);

    // List available hardware device types
    println!("Available hardware device types:");
    for hw_type in AVHWDeviceType::iter_types() {
        if let Some(name) = hw_type.name() {
            println!("  - {}", name);
        }
    }

    // Open input file
    let mut input = Input::open(input_path)?;
    let streams = input.streams();

    // Find video stream
    let video_stream = streams.best(AVMediaType::Video).ok_or("No video stream found")?;

    println!("Video stream info:");
    println!("  Index: {}", video_stream.index());
    println!("  Codec: {:?}", video_stream.codec_parameters().map(|p| p.codec_id));
    println!("  Time base: {:?}", video_stream.time_base());

    // Create decoder with hardware acceleration
    let decoder_options = DecoderOptions::default().with_hardware(hardware_config.clone());

    let mut video_decoder = match Decoder::with_options(&video_stream, decoder_options) {
        Ok(decoder) => decoder.video().map_err(|_| "Failed to get video decoder")?,
        Err(e) => {
            eprintln!("Failed to create hardware decoder: {}", e);

            // Fallback to software decoding
            println!("Falling back to software decoding...");
            let software_options = DecoderOptions::default();
            Decoder::with_options(&video_stream, software_options)?
                .video()
                .map_err(|_| "Failed to get video decoder")?
        }
    };

    println!("Decoder info:");
    println!("  Width: {}", video_decoder.width());
    println!("  Height: {}", video_decoder.height());
    println!("  Pixel format: {:?}", video_decoder.pixel_format());
    println!("  Frame rate: {:?}", video_decoder.frame_rate());

    // Create hardware context for manual frame transfer (if needed)
    let hardware_context = if hardware_config.is_hardware_accelerated() {
        match HardwareContext::new(hardware_config.device_type, hardware_config.device.as_deref()) {
            Ok(ctx) => {
                println!("Hardware context created successfully");
                Some(ctx)
            }
            Err(e) => {
                eprintln!("Failed to create hardware context: {}", e);
                None
            }
        }
    } else {
        None
    };

    let video_stream_index = video_stream.index();
    let mut frame_count = 0;
    let mut hw_frame_count = 0;

    // Process packets
    while let Some(packet) = input.receive_packet()? {
        if packet.stream_index() == video_stream_index {
            video_decoder.send_packet(&packet)?;

            // Receive frames
            while let Some(frame) = video_decoder.receive_frame()? {
                frame_count += 1;

                // Check if it's a hardware frame
                if let Some(ref hw_ctx) = hardware_context {
                    if hw_ctx.is_hw_frame(&frame) {
                        hw_frame_count += 1;
                        println!(
                            "Frame {}: Hardware frame ({}x{}, format: {:?})",
                            frame_count,
                            frame.width(),
                            frame.height(),
                            frame.format()
                        );

                        // Example: Manual transfer to system memory
                        if !hardware_config.auto_transfer {
                            let sw_frame = hw_ctx.transfer_data_from_hw(&frame)?;
                            println!("  Transferred to system memory: format: {:?}", sw_frame.format());
                        }
                    } else {
                        println!(
                            "Frame {}: Software frame ({}x{}, format: {:?})",
                            frame_count,
                            frame.width(),
                            frame.height(),
                            frame.format()
                        );
                    }
                } else {
                    println!(
                        "Frame {}: Software frame ({}x{}, format: {:?})",
                        frame_count,
                        frame.width(),
                        frame.height(),
                        frame.format()
                    );
                }

                // Limit output for demo purposes
                if frame_count >= 10 {
                    break;
                }
            }
        }

        if frame_count >= 10 {
            break;
        }
    }

    // Flush decoder
    video_decoder.send_eof()?;
    while let Some(frame) = video_decoder.receive_frame()? {
        frame_count += 1;

        if let Some(ref hw_ctx) = hardware_context {
            if hw_ctx.is_hw_frame(&frame) {
                hw_frame_count += 1;
            }
        }

        if frame_count >= 10 {
            break;
        }
    }

    println!("\nDecoding summary:");
    println!("  Total frames decoded: {}", frame_count);
    println!("  Hardware frames: {}", hw_frame_count);
    println!("  Software frames: {}", frame_count - hw_frame_count);

    if hardware_config.is_hardware_accelerated() {
        println!("  Hardware acceleration: {}", hardware_config.device_type);
        println!("  Auto transfer: {}", hardware_config.auto_transfer);
    } else {
        println!("  Hardware acceleration: disabled");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_device_types() {
        // Test that we can iterate over hardware device types
        let types: Vec<_> = AVHWDeviceType::iter_types().collect();
        assert!(!types.is_empty());

        // Test that we can find CUDA if available
        if let Some(cuda_type) = AVHWDeviceType::find_by_name("cuda") {
            assert_eq!(cuda_type, AVHWDeviceType::Cuda);
        }
    }

    #[test]
    fn test_hardware_config_creation() {
        let cuda_config = HardwareConfig::cuda();
        assert_eq!(cuda_config.device_type, AVHWDeviceType::Cuda);
        assert!(cuda_config.auto_transfer);
        assert!(cuda_config.is_hardware_accelerated());

        let vaapi_config = HardwareConfig::vaapi();
        assert_eq!(vaapi_config.device_type, AVHWDeviceType::Vaapi);
        assert!(vaapi_config.auto_transfer);
        assert!(vaapi_config.is_hardware_accelerated());

        let software_config = HardwareConfig::default();
        assert_eq!(software_config.device_type, AVHWDeviceType::None);
        assert!(!software_config.is_hardware_accelerated());
    }
}
