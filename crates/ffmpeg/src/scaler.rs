use crate::AVPixelFormat;
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::frame::VideoFrame;
use crate::smart_object::SmartPtr;

/// Scaling algorithm to use for video scaling operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingAlgorithm {
    /// Fast bilinear scaling algorithm (fastest, lower quality)
    FastBilinear,
    /// Bilinear scaling algorithm (good balance of speed and quality)
    Bilinear,
    /// Bicubic scaling algorithm (higher quality, slower)
    Bicubic,
    /// Experimental scaling algorithm
    Experimental,
    /// Nearest neighbor scaling algorithm (fastest, pixelated result)
    Point,
    /// Area averaging scaling algorithm (good for downscaling)
    Area,
    /// Bicubic for luma, bilinear for chroma
    Bicublin,
    /// Gaussian scaling algorithm
    Gauss,
    /// Sinc scaling algorithm
    Sinc,
    /// Lanczos scaling algorithm (high quality, good for upscaling)
    Lanczos,
    /// Natural bicubic spline scaling algorithm
    Spline,
}

impl ScalingAlgorithm {
    /// Returns the FFmpeg flag value for this scaling algorithm.
    pub const fn as_flags(self) -> i32 {
        match self {
            Self::FastBilinear => SWS_FAST_BILINEAR as i32,
            Self::Bilinear => SWS_BILINEAR as i32,
            Self::Bicubic => SWS_BICUBIC as i32,
            Self::Experimental => SWS_X as i32,
            Self::Point => SWS_POINT as i32,
            Self::Area => SWS_AREA as i32,
            Self::Bicublin => SWS_BICUBLIN as i32,
            Self::Gauss => SWS_GAUSS as i32,
            Self::Sinc => SWS_SINC as i32,
            Self::Lanczos => SWS_LANCZOS as i32,
            Self::Spline => SWS_SPLINE as i32,
        }
    }
}

/// A scaler is a wrapper around an [`SwsContext`]. Which is used to scale or transform video frames.
///
/// The scaler supports various scaling algorithms that can be selected based on the desired
/// trade-off between quality and performance:
///
/// - [`ScalingAlgorithm::FastBilinear`]: Fastest, lower quality
/// - [`ScalingAlgorithm::Bilinear`]: Good balance (default)
/// - [`ScalingAlgorithm::Bicubic`]: Higher quality, slower
/// - [`ScalingAlgorithm::Lanczos`]: High quality, excellent for upscaling
/// - [`ScalingAlgorithm::Area`]: Good for downscaling
/// - [`ScalingAlgorithm::Point`]: Nearest neighbor, fastest but pixelated
///
/// # Examples
///
/// ```rust
/// use scuffle_ffmpeg::scaler::{VideoScaler, ScalingAlgorithm};
/// use scuffle_ffmpeg::AVPixelFormat;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a scaler that converts 1920x1080 YUV420P to 1280x720 RGB24
/// let mut scaler = VideoScaler::with_algorithm(
///     1920, 1080, AVPixelFormat::Yuv420p,
///     1280, 720, AVPixelFormat::Rgb24,
///     ScalingAlgorithm::Lanczos
/// )?;
///
/// // Process frames through the scaler
/// // let output_frame = scaler.process(&input_frame)?;
/// # Ok(())
/// # }
/// ```
pub struct VideoScaler {
    ptr: SmartPtr<SwsContext>,
    frame: VideoFrame,
    pixel_format: AVPixelFormat,
    width: i32,
    height: i32,
    algorithm: ScalingAlgorithm,
}

/// Safety: `Scaler` is safe to send between threads.
unsafe impl Send for VideoScaler {}

impl VideoScaler {
    /// Creates a new `VideoScaler` instance with the default bilinear algorithm.
    pub fn new(
        input_width: i32,
        input_height: i32,
        incoming_pixel_fmt: AVPixelFormat,
        width: i32,
        height: i32,
        pixel_format: AVPixelFormat,
    ) -> Result<Self, FfmpegError> {
        Self::with_algorithm(
            input_width,
            input_height,
            incoming_pixel_fmt,
            width,
            height,
            pixel_format,
            ScalingAlgorithm::Bilinear,
        )
    }

    /// Creates a new `VideoScaler` instance with the specified scaling algorithm.
    pub fn with_algorithm(
        input_width: i32,
        input_height: i32,
        incoming_pixel_fmt: AVPixelFormat,
        width: i32,
        height: i32,
        pixel_format: AVPixelFormat,
        algorithm: ScalingAlgorithm,
    ) -> Result<Self, FfmpegError> {
        // Safety: `sws_getContext` is safe to call, and the pointer returned is valid.
        let ptr = unsafe {
            sws_getContext(
                input_width,
                input_height,
                incoming_pixel_fmt.into(),
                width,
                height,
                pixel_format.into(),
                algorithm.as_flags(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };

        let destructor = |ptr: &mut *mut SwsContext| {
            // Safety: `sws_freeContext` is safe to call.
            unsafe {
                sws_freeContext(*ptr);
            }

            *ptr = std::ptr::null_mut();
        };

        // Safety: `ptr` is a valid pointer & `destructor` has been setup to free the context.
        let ptr = unsafe { SmartPtr::wrap_non_null(ptr, destructor) }.ok_or(FfmpegError::Alloc)?;

        let frame = VideoFrame::builder()
            .width(width)
            .height(height)
            .pix_fmt(pixel_format)
            .build()?;

        Ok(Self {
            ptr,
            frame,
            pixel_format,
            width,
            height,
            algorithm,
        })
    }

    /// Returns the pixel format of the scaler.
    pub const fn pixel_format(&self) -> AVPixelFormat {
        self.pixel_format
    }

    /// Returns the width of the scaler.
    pub const fn width(&self) -> i32 {
        self.width
    }

    /// Returns the height of the scaler.
    pub const fn height(&self) -> i32 {
        self.height
    }

    /// Returns the scaling algorithm used by the scaler.
    pub const fn algorithm(&self) -> ScalingAlgorithm {
        self.algorithm
    }

    /// Processes a frame through the scaler.
    pub fn process<'a>(&'a mut self, frame: &VideoFrame) -> Result<&'a VideoFrame, FfmpegError> {
        // Safety: `frame` is a valid pointer, and `self.ptr` is a valid pointer.
        let frame_ptr = unsafe { frame.as_ptr().as_ref().unwrap() };
        // Safety: `self.frame` is a valid pointer.
        let self_frame_ptr = unsafe { self.frame.as_ptr().as_ref().unwrap() };

        // Safety: `sws_scale` is safe to call.
        FfmpegErrorCode(unsafe {
            sws_scale(
                self.ptr.as_mut_ptr(),
                frame_ptr.data.as_ptr() as *const *const u8,
                frame_ptr.linesize.as_ptr(),
                0,
                frame_ptr.height,
                self_frame_ptr.data.as_ptr(),
                self_frame_ptr.linesize.as_ptr(),
            )
        })
        .result()?;

        // Copy the other fields from the input frame to the output frame.
        self.frame.set_dts(frame.dts());
        self.frame.set_pts(frame.pts());
        self.frame.set_duration(frame.duration());
        self.frame.set_time_base(frame.time_base());

        Ok(&self.frame)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use insta::assert_debug_snapshot;
    use rand::Rng;

    use crate::frame::VideoFrame;
    use crate::scaler::{AVPixelFormat, ScalingAlgorithm, VideoScaler};

    #[test]
    fn test_scaler_new() {
        let input_width = 1920;
        let input_height = 1080;
        let incoming_pixel_fmt = AVPixelFormat::Yuv420p;
        let output_width = 1280;
        let output_height = 720;
        let output_pixel_fmt = AVPixelFormat::Rgb24;
        let scaler = VideoScaler::new(
            input_width,
            input_height,
            incoming_pixel_fmt,
            output_width,
            output_height,
            output_pixel_fmt,
        );

        assert!(scaler.is_ok(), "Expected VideoScaler::new to succeed");
        let scaler = scaler.unwrap();

        assert_eq!(
            scaler.width(),
            output_width,
            "Expected VideoScaler width to match the output width"
        );
        assert_eq!(
            scaler.height(),
            output_height,
            "Expected VideoScaler height to match the output height"
        );
        assert_eq!(
            scaler.pixel_format(),
            output_pixel_fmt,
            "Expected VideoScaler pixel format to match the output pixel format"
        );
        assert_eq!(
            scaler.algorithm(),
            ScalingAlgorithm::Bilinear,
            "Expected VideoScaler to use default bilinear algorithm"
        );
    }

    #[test]
    fn test_scaler_with_algorithm() {
        let input_width = 1920;
        let input_height = 1080;
        let incoming_pixel_fmt = AVPixelFormat::Yuv420p;
        let output_width = 1280;
        let output_height = 720;
        let output_pixel_fmt = AVPixelFormat::Rgb24;
        let algorithm = ScalingAlgorithm::Lanczos;

        let scaler = VideoScaler::with_algorithm(
            input_width,
            input_height,
            incoming_pixel_fmt,
            output_width,
            output_height,
            output_pixel_fmt,
            algorithm,
        );

        assert!(scaler.is_ok(), "Expected VideoScaler::with_algorithm to succeed");
        let scaler = scaler.unwrap();

        assert_eq!(
            scaler.algorithm(),
            algorithm,
            "Expected VideoScaler to use specified algorithm"
        );
        assert_eq!(
            scaler.width(),
            output_width,
            "Expected VideoScaler width to match the output width"
        );
        assert_eq!(
            scaler.height(),
            output_height,
            "Expected VideoScaler height to match the output height"
        );
        assert_eq!(
            scaler.pixel_format(),
            output_pixel_fmt,
            "Expected VideoScaler pixel format to match the output pixel format"
        );
    }

    #[test]
    fn test_scaler_process() {
        let input_width = 1920;
        let input_height = 1080;
        let incoming_pixel_fmt = AVPixelFormat::Yuv420p;
        let output_width = 1280;
        let output_height = 720;
        let output_pixel_fmt = AVPixelFormat::Rgb24;

        let mut scaler = VideoScaler::new(
            input_width,
            input_height,
            incoming_pixel_fmt,
            output_width,
            output_height,
            output_pixel_fmt,
        )
        .expect("Failed to create VideoScaler");

        let mut input_frame = VideoFrame::builder()
            .width(input_width)
            .height(input_height)
            .pix_fmt(incoming_pixel_fmt)
            .build()
            .expect("Failed to create VideoFrame");

        // We need to fill the buffer with random data otherwise the result will be based off uninitialized data.
        let mut rng = rand::rng();

        for data_idx in 0..rusty_ffmpeg::ffi::AV_NUM_DATA_POINTERS {
            if let Some(mut data_buf) = input_frame.data_mut(data_idx as usize) {
                for row_idx in 0..data_buf.height() {
                    let row = data_buf.get_row_mut(row_idx as usize).unwrap();
                    rng.fill(row);
                }
            }
        }

        let result = scaler.process(&input_frame);

        assert!(
            result.is_ok(),
            "Expected VideoScaler::process to succeed, but got error: {result:?}"
        );

        let output_frame = result.unwrap();
        assert_debug_snapshot!(output_frame, @r"
        VideoFrame {
            width: 1280,
            height: 720,
            sample_aspect_ratio: Rational {
                numerator: 1,
                denominator: 1,
            },
            pts: None,
            dts: None,
            duration: Some(
                0,
            ),
            best_effort_timestamp: None,
            time_base: Rational {
                numerator: 0,
                denominator: 1,
            },
            format: AVPixelFormat::Rgb24,
            is_audio: false,
            is_video: true,
            is_keyframe: false,
        }
        ");
    }
}
