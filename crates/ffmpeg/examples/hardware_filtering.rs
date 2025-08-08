//! Example demonstrating hardware-accelerated filtering with CUDA
//!
//! This example shows how to set up hardware frames context for GPU-accelerated
//! filtering using CUDA. This is useful when you want to process video frames
//! that are already on the GPU without transferring them to CPU memory.

use scuffle_ffmpeg::filter_graph::{FilterGraph, Filter, HWFramesContext};
use scuffle_ffmpeg::{AVPixelFormat, error::FfmpegError};
use scuffle_ffmpeg::ffi::*;


/// Example function showing how to set up hardware-accelerated filtering
///
/// # Arguments
/// * `hw_device_ctx` - A valid CUDA hardware device context (AVBufferRef)
/// * `input_width` - Width of input frames
/// * `input_height` - Height of input frames
///
/// # Safety
/// The `hw_device_ctx` must be a valid, initialized CUDA device context.
pub unsafe fn setup_cuda_filter_chain(
    hw_device_ctx: *mut AVBufferRef,
    input_width: i32,
    input_height: i32,
) -> Result<FilterGraph, FfmpegError> {
    // Create a new filter graph
    let mut filter_graph = FilterGraph::new()?;

    // Add a buffer source filter for hardware frames
    let buffer_filter = Filter::get("buffer").ok_or(FfmpegError::NoFilter)?;
    let buffer_args = format!(
        "width={}:height={}:pix_fmt={}:time_base=1/25",
        input_width,
        input_height,
        i32::from(AVPixelFormat::Cuda)
    );
    filter_graph.add(buffer_filter, "hw_buffer", &buffer_args)?;

    // Add a scale_cuda filter for GPU-based scaling
    let scale_cuda_filter = Filter::get("scale_cuda").ok_or(FfmpegError::NoFilter)?;
    filter_graph.add(scale_cuda_filter, "scale", "1280:720")?;

    // Add a buffersink for output
    let buffersink_filter = Filter::get("buffersink").ok_or(FfmpegError::NoFilter)?;
    filter_graph.add(buffersink_filter, "sink", "")?;

    // Link the filters together
    filter_graph.link("hw_buffer", 0, "scale", 0)?;
    filter_graph.link("scale", 0, "sink", 0)?;

    // Create hardware frames context for the input
    let hw_frames = unsafe {
        HWFramesContext::new_cuda(
            hw_device_ctx,
            input_width,
            input_height,
            Some(8), // Pool size of 8 frames
        )?
    };

    // Set the hardware frames context on the buffer source
    let mut source_context = filter_graph.get("hw_buffer").ok_or(FfmpegError::NoFilter)?.source();
    source_context.set_hw_frames_context(&hw_frames)?;

    // Validate the filter graph
    filter_graph.validate()?;

    Ok(filter_graph)
}

/// Example showing manual hardware frames context setup
///
/// This demonstrates the lower-level API for more control over the hardware context parameters.
pub unsafe fn setup_custom_hw_frames_context(
    hw_device_ctx: *mut AVBufferRef,
) -> Result<HWFramesContext, FfmpegError> {
    // Create hardware frames context with custom parameters
    let hw_frames = unsafe {
        HWFramesContext::new(
            hw_device_ctx,
            AVPixelFormat::Cuda,        // Hardware format (GPU memory)
            AVPixelFormat::Nv12,        // Software format (for CPU fallback)
            3840,                       // 4K width
            2160,                       // 4K height
            16,                         // Larger pool for 4K processing
        )?
    };

    Ok(hw_frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_compiles() {
        // This test just verifies that the example functions compile correctly
        // We can't actually run them without proper CUDA setup

        let _setup_fn = setup_cuda_filter_chain;
        let _custom_fn = setup_custom_hw_frames_context;

        assert!(true, "Example functions compile successfully");
    }
}

// Note: To actually use this in a real application, you would need to:
//
// 1. Initialize CUDA device context:
//    ```c
//    AVBufferRef *hw_device_ctx = NULL;
//    av_hwdevice_ctx_create(&hw_device_ctx, AV_HWDEVICE_TYPE_CUDA, NULL, NULL, 0);
//    ```
//
// 2. Create your filter graph with hardware support:
//    ```rust
//    let filter_graph = unsafe { setup_cuda_filter_chain(hw_device_ctx, 1920, 1080)? };
//    ```
//
// 3. Process frames through the hardware-accelerated filter chain:
//    ```rust
//    let mut source = filter_graph.get("hw_buffer")?.source();
//    let mut sink = filter_graph.get("sink")?.sink();
//
//    // Send GPU frames to the filter
//    source.send_frame(&gpu_frame)?;
//
//    // Receive processed frames
//    while let Some(processed_frame) = sink.receive_frame()? {
//        // Handle the processed frame
//    }
//    ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hardware filtering example");
    println!("This example demonstrates the API for hardware-accelerated filtering with CUDA.");
    println!("To actually use this functionality, you would need:");
    println!("1. A properly initialized CUDA device context");
    println!("2. GPU frames in CUDA memory");
    println!("3. CUDA-enabled FFmpeg build");

    Ok(())
}
