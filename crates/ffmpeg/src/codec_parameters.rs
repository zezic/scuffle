use std::ptr::NonNull;

use crate::encoder::Encoder;
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::rational::Rational;
use crate::smart_object::SmartPtr;
use crate::stream::Stream;

/// An owned set of codec parameters that can be extracted from a stream
/// and later used to create a new stream in an output context.
///
/// This is useful for remuxing scenarios where packets are transferred
/// from input to output without transcoding.
pub struct CodecParameters {
    ptr: SmartPtr<AVCodecParameters>,
    time_base: Rational,
    start_time: Option<i64>,
    duration: Option<i64>,
}

/// Safety: `CodecParameters` is safe to send between threads.
unsafe impl Send for CodecParameters {}

/// Safety: `CodecParameters` is safe to share between threads.
unsafe impl Sync for CodecParameters {}

impl CodecParameters {
    /// Creates a new `CodecParameters` by copying the codec parameters from the given stream.
    pub fn from_stream(stream: &Stream<'_>) -> Result<Self, FfmpegError> {
        let codec_param = stream
            .codec_parameters()
            .ok_or(FfmpegError::Arguments("stream has no codec parameters"))?;

        // Safety: `avcodec_parameters_alloc` is safe to call.
        let ptr = NonNull::new(unsafe { avcodec_parameters_alloc() }).ok_or(FfmpegError::Alloc)?;

        // Safety: The pointer is valid and we own it.
        let mut ptr = unsafe {
            SmartPtr::wrap(ptr.as_ptr(), |ptr| {
                avcodec_parameters_free(ptr);
            })
        };

        // Safety: `avcodec_parameters_copy` is safe to call when both pointers are valid.
        FfmpegErrorCode(unsafe { avcodec_parameters_copy(ptr.as_mut_ptr(), codec_param) }).result()?;

        Ok(Self {
            ptr,
            time_base: stream.time_base(),
            start_time: stream.start_time(),
            duration: stream.duration(),
        })
    }

    /// Creates a new `CodecParameters` by reading the codec parameters from the given encoder.
    /// If a `stream` is provided, the start time and duration are copied from it.
    pub fn from_encoder(encoder: &Encoder, stream: Option<&Stream<'_>>) -> Result<Self, FfmpegError> {
        // Safety: `avcodec_parameters_alloc` is safe to call.
        let ptr = NonNull::new(unsafe { avcodec_parameters_alloc() }).ok_or(FfmpegError::Alloc)?;

        // Safety: The pointer is valid and we own it.
        let mut ptr = unsafe {
            SmartPtr::wrap(ptr.as_ptr(), |ptr| {
                avcodec_parameters_free(ptr);
            })
        };

        // Safety: `avcodec_parameters_from_context` is safe to call when both pointers are valid.
        FfmpegErrorCode(unsafe { avcodec_parameters_from_context(ptr.as_mut_ptr(), encoder.codec_context()) }).result()?;

        Ok(Self {
            ptr,
            time_base: encoder.outgoing_time_base(),
            start_time: stream.and_then(|s| s.start_time()),
            duration: stream.and_then(|s| s.duration()),
        })
    }

    /// Returns a pointer to the underlying `AVCodecParameters`.
    pub const fn as_ptr(&self) -> *const AVCodecParameters {
        self.ptr.as_ptr()
    }

    /// Returns the time base associated with these codec parameters.
    pub const fn time_base(&self) -> Rational {
        self.time_base
    }

    /// Sets the time base associated with these codec parameters.
    pub fn set_time_base(&mut self, time_base: impl Into<Rational>) {
        self.time_base = time_base.into();
    }

    /// Returns the start time associated with these codec parameters.
    pub const fn start_time(&self) -> Option<i64> {
        self.start_time
    }

    /// Returns the duration associated with these codec parameters.
    pub const fn duration(&self) -> Option<i64> {
        self.duration
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use crate::codec::EncoderCodec;
    use crate::encoder::{Encoder, VideoEncoderSettings};
    use crate::io::{Input, Output, OutputOptions};
    use crate::{AVCodecID, AVMediaType, AVPixelFormat};

    use super::*;

    #[test]
    fn test_from_stream() {
        let input = Input::open("../../assets/avc_aac_large.mp4").expect("Failed to open input");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("no video stream");

        let params = CodecParameters::from_stream(&video_stream).expect("Failed to extract codec parameters");

        assert!(!params.as_ptr().is_null());
        assert_eq!(params.time_base(), video_stream.time_base());
        assert_eq!(params.start_time(), video_stream.start_time());
        assert_eq!(params.duration(), video_stream.duration());
    }

    #[test]
    fn test_from_audio_stream() {
        let input = Input::open("../../assets/avc_aac_large.mp4").expect("Failed to open input");
        let streams = input.streams();
        let audio_stream = streams.best(AVMediaType::Audio).expect("no audio stream");

        let params = CodecParameters::from_stream(&audio_stream).expect("Failed to extract codec parameters");

        assert!(!params.as_ptr().is_null());
        assert_eq!(params.time_base(), audio_stream.time_base());
    }

    #[test]
    fn test_from_encoder() {
        let codec = EncoderCodec::new(AVCodecID::Mpeg4).expect("Failed to find MPEG-4 encoder");
        let data = std::io::Cursor::new(Vec::new());
        let options = OutputOptions::builder().format_name("mp4").unwrap().build();
        let mut output = Output::new(data, options).expect("Failed to create Output");
        let incoming_time_base = Rational::static_new::<1, 1000>();
        let outgoing_time_base = Rational::static_new::<1, 30>();
        let settings = VideoEncoderSettings::builder()
            .width(640)
            .height(480)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .build();
        let encoder = Encoder::new(codec, &mut output, incoming_time_base, outgoing_time_base, settings)
            .expect("Failed to create encoder");

        let params = CodecParameters::from_encoder(&encoder, None).expect("Failed to extract codec parameters from encoder");

        assert!(!params.as_ptr().is_null());
        assert_eq!(params.time_base(), outgoing_time_base);
        assert_eq!(params.start_time(), None);
        assert_eq!(params.duration(), None);
    }

    #[test]
    fn test_from_encoder_with_stream() {
        let input = Input::open("../../assets/avc_aac_large.mp4").expect("Failed to open input");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("no video stream");

        let codec = EncoderCodec::new(AVCodecID::Mpeg4).expect("Failed to find MPEG-4 encoder");
        let data = std::io::Cursor::new(Vec::new());
        let options = OutputOptions::builder().format_name("mp4").unwrap().build();
        let mut output = Output::new(data, options).expect("Failed to create Output");
        let incoming_time_base = Rational::static_new::<1, 1000>();
        let outgoing_time_base = Rational::static_new::<1, 30>();
        let settings = VideoEncoderSettings::builder()
            .width(640)
            .height(480)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .build();
        let encoder = Encoder::new(codec, &mut output, incoming_time_base, outgoing_time_base, settings)
            .expect("Failed to create encoder");

        let params = CodecParameters::from_encoder(&encoder, Some(&video_stream))
            .expect("Failed to extract codec parameters from encoder");

        assert!(!params.as_ptr().is_null());
        assert_eq!(params.time_base(), outgoing_time_base);
        assert_eq!(params.start_time(), video_stream.start_time());
        assert_eq!(params.duration(), video_stream.duration());
    }
}
