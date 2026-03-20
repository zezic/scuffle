use core::num::NonZero;

use crate::codec::DecoderCodec;
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::frame::{AudioFrame, GenericFrame, VideoFrame};
use crate::packet::Packet;
use crate::rational::Rational;
use crate::smart_object::SmartPtr;
use crate::stream::Stream;
use crate::{AVCodecID, AVMediaType, AVPixelFormat, AVSampleFormat};

/// Either a [`VideoDecoder`] or an [`AudioDecoder`].
///
/// This is the most common way to interact with decoders.
#[derive(Debug)]
pub enum Decoder {
    /// A video decoder.
    Video(VideoDecoder),
    /// An audio decoder.
    Audio(AudioDecoder),
}

/// A generic decoder that can be used to decode any type of media.
pub struct GenericDecoder {
    decoder: SmartPtr<AVCodecContext>,
}

/// Safety: `GenericDecoder` can be sent between threads.
unsafe impl Send for GenericDecoder {}

impl std::fmt::Debug for GenericDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder")
            .field("time_base", &self.time_base())
            .field("codec_type", &self.codec_type())
            .finish()
    }
}

/// A video decoder.
/// Video decoder wrapper around GenericDecoder
pub struct VideoDecoder(GenericDecoder);

impl std::fmt::Debug for VideoDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoDecoder")
            .field("time_base", &self.time_base())
            .field("width", &self.width())
            .field("height", &self.height())
            .field("pixel_format", &self.pixel_format())
            .field("frame_rate", &self.frame_rate())
            .field("sample_aspect_ratio", &self.sample_aspect_ratio())
            .finish()
    }
}

/// An audio decoder.
pub struct AudioDecoder(GenericDecoder);

impl std::fmt::Debug for AudioDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDecoder")
            .field("time_base", &self.time_base())
            .field("sample_rate", &self.sample_rate())
            .field("channels", &self.channels())
            .field("sample_fmt", &self.sample_format())
            .finish()
    }
}

/// Hardware acceleration options for a [`Decoder`].
pub struct HardwareAccelerationOptions {
    /// The hardware acceleration device type to use for decoding.
    pub hw_device_type: AVHWDeviceType,
    /// The pixel format to use for hardware acceleration.
    pub hw_pixel_format: AVPixelFormat,
}

/// Controls which multithreading method the decoder uses.
///
/// Frame threading (`Frame`) decodes multiple frames in parallel but adds
/// `thread_count - 1` frames of latency.  Slice threading (`Slice`) decodes
/// slices of a single frame in parallel with **no** additional latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadType {
    /// Decode different frames in parallel (adds latency).
    Frame,
    /// Decode different slices of a single frame in parallel (no extra latency).
    Slice,
    /// Allow both frame and slice threading (FFmpeg default).
    FrameAndSlice,
}

/// Options for creating a [`Decoder`].
pub struct DecoderOptions {
    /// The codec to use for decoding.
    pub codec: Option<DecoderCodec>,
    /// The number of threads to use for decoding.
    pub thread_count: i32,
    /// Which multithreading method to use. Defaults to `FrameAndSlice`.
    ///
    /// For low-latency decoding, use [`ThreadType::Slice`] to get parallelism
    /// without the frame-threading latency penalty.
    pub thread_type: ThreadType,
    /// Force low-delay mode (`AV_CODEC_FLAG_LOW_DELAY`).
    ///
    /// When enabled this disables frame threading entirely and signals to the
    /// codec that low-delay output is desired.  Combine with
    /// [`ThreadType::Slice`] and a suitable `thread_count` to get parallel
    /// decoding without buffering extra frames.
    pub low_delay: bool,
    /// The hardware acceleration options to use for decoding.
    pub hardware_acceleration: Option<HardwareAccelerationOptions>,
}

/// The default options for a [`Decoder`].
impl Default for DecoderOptions {
    fn default() -> Self {
        Self {
            codec: None,
            thread_count: 1,
            thread_type: ThreadType::FrameAndSlice,
            low_delay: false,
            hardware_acceleration: None,
        }
    }
}

impl Decoder {
    /// Creates a new [`Decoder`] with the default options.
    pub fn new(ist: &Stream) -> Result<Self, FfmpegError> {
        Self::with_options(ist, Default::default())
    }

    /// Creates a new [`Decoder`] with the given options.
    pub fn with_options(ist: &Stream, options: DecoderOptions) -> Result<Self, FfmpegError> {
        let Some(codec_params) = ist.codec_parameters() else {
            return Err(FfmpegError::NoDecoder);
        };

        let codec = options
            .codec
            .or_else(|| DecoderCodec::new(AVCodecID(codec_params.codec_id as _)))
            .ok_or(FfmpegError::NoDecoder)?;

        if codec.is_empty() {
            return Err(FfmpegError::NoDecoder);
        }

        // Safety: `avcodec_alloc_context3` is safe to call and all arguments are valid.
        let decoder = unsafe { avcodec_alloc_context3(codec.as_ptr()) };

        let destructor = |ptr: &mut *mut AVCodecContext| {
            // Clean up opaque field if it was used for hardware acceleration
            unsafe {
                if !ptr.is_null() && !(**ptr).opaque.is_null() {
                    let _ = Box::from_raw((**ptr).opaque as *mut AVPixelFormat);
                }
                // Safety: The pointer here is valid.
                avcodec_free_context(ptr);
            }
        };

        // Safety: `decoder` is a valid pointer, and `destructor` has been setup to free the context.
        let mut decoder = unsafe { SmartPtr::wrap_non_null(decoder, destructor) }.ok_or(FfmpegError::Alloc)?;

        // Safety: `codec_params` is a valid pointer, and `decoder` is a valid pointer.
        FfmpegErrorCode(unsafe { avcodec_parameters_to_context(decoder.as_mut_ptr(), codec_params) }).result()?;

        let decoder_mut = decoder.as_deref_mut_except();

        decoder_mut.pkt_timebase = ist.time_base().into();
        decoder_mut.time_base = ist.time_base().into();
        decoder_mut.thread_count = options.thread_count;
        decoder_mut.thread_type = match options.thread_type {
            ThreadType::Frame => FF_THREAD_FRAME as i32,
            ThreadType::Slice => FF_THREAD_SLICE as i32,
            ThreadType::FrameAndSlice => (FF_THREAD_FRAME | FF_THREAD_SLICE) as i32,
        };
        if options.low_delay {
            decoder_mut.flags |= AV_CODEC_FLAG_LOW_DELAY as i32;
        }

        if let Some(hw_accel) = options.hardware_acceleration {
            let HardwareAccelerationOptions {
                hw_device_type,
                hw_pixel_format,
            } = hw_accel;
            // Store the preferred format in the decoder's opaque field so the callback can access it
            decoder_mut.opaque = Box::into_raw(Box::new(hw_pixel_format)) as *mut std::ffi::c_void;

            decoder_mut.get_format = Some(get_hw_format);

            let mut hw_device_ctx = std::ptr::null_mut();
            let ptr = &mut hw_device_ctx;

            FfmpegErrorCode(unsafe {
                av_hwdevice_ctx_create(ptr, hw_device_type, std::ptr::null(), std::ptr::null_mut(), 0)
            })
            .result()?;

            decoder_mut.hw_device_ctx = hw_device_ctx;

            // Note: we do NOT set hw_frames_ctx on the decoder here.
            // With custom I/O readers, VT auto-transfers frames to CPU (yuv420p).
            // The TransferThenScale scaler handles both GPU and CPU frames at runtime.
        }

        if AVMediaType(decoder_mut.codec_type) == AVMediaType::Video {
            // Safety: Even though we are upcasting `AVFormatContext` from a const pointer to a
            // mutable pointer, it is still safe becasuse av_guess_frame_rate does not use
            // the pointer to modify the `AVFormatContext`. https://github.com/FFmpeg/FFmpeg/blame/268d0b6527cba1ebac1f44347578617341f85c35/libavformat/avformat.c#L763
            // The function does not use the pointer at all, it only uses the `AVStream`
            // pointer to get the `AVRational`
            let format_context = unsafe { ist.format_context() };

            decoder_mut.framerate =
                // Safety: See above.
                unsafe { av_guess_frame_rate(format_context, ist.as_ptr() as *mut AVStream, std::ptr::null_mut()) };
        }

        if matches!(AVMediaType(decoder_mut.codec_type), AVMediaType::Video | AVMediaType::Audio) {
            // Safety: `codec` is a valid pointer, and `decoder` is a valid pointer.
            FfmpegErrorCode(unsafe { avcodec_open2(decoder_mut, codec.as_ptr(), std::ptr::null_mut()) }).result()?;
        }

        Ok(match AVMediaType(decoder_mut.codec_type) {
            AVMediaType::Video => Self::Video(VideoDecoder(GenericDecoder { decoder })),
            AVMediaType::Audio => Self::Audio(AudioDecoder(GenericDecoder { decoder })),
            _ => Err(FfmpegError::NoDecoder)?,
        })
    }

    /// Returns the video decoder if the decoder is a video decoder.
    pub fn video(self) -> Result<VideoDecoder, Self> {
        match self {
            Self::Video(video) => Ok(video),
            _ => Err(self),
        }
    }

    /// Returns the audio decoder if the decoder is an audio decoder.
    pub fn audio(self) -> Result<AudioDecoder, Self> {
        match self {
            Self::Audio(audio) => Ok(audio),
            _ => Err(self),
        }
    }
}

unsafe extern "C" fn get_hw_format(ctx: *mut AVCodecContext, pix_fmts: *const i32) -> i32 {
    if ctx.is_null() || pix_fmts.is_null() {
        return AVPixelFormat::None.into();
    }

    // Retrieve the preferred format from the opaque field
    let preferred_format = if !unsafe { *ctx }.opaque.is_null() {
        unsafe { *((*ctx).opaque as *const AVPixelFormat) }
    } else {
        return AVPixelFormat::None.into(); // AV_PIX_FMT_NONE
    };

    // Iterate through available formats
    let mut p = pix_fmts;
    let mut first_format = None;
    loop {
        let current_format = AVPixelFormat::from(unsafe { *p });
        if current_format == AVPixelFormat::None {
            break;
        }
        if first_format.is_none() {
            first_format = Some(current_format);
        }
        if current_format == preferred_format {
            return current_format.into();
        }
        p = unsafe { p.add(1) };
    }

    // HW format not available — return first software format so decoding still works
    let fmt: i32 = first_format.unwrap_or(AVPixelFormat::None).into();
    fmt
}

impl GenericDecoder {
    /// Returns the codec type of the decoder.
    pub const fn codec_type(&self) -> AVMediaType {
        AVMediaType(self.decoder.as_deref_except().codec_type)
    }

    /// Returns the time base of the decoder or `None` if the denominator is zero.
    pub const fn time_base(&self) -> Option<Rational> {
        let time_base = self.decoder.as_deref_except().time_base;
        match NonZero::new(time_base.den) {
            Some(den) => Some(Rational::new(time_base.num, den)),
            None => None,
        }
    }

    /// Returns the hardware device context buffer reference pointer.
    /// Returns `None` if no hardware acceleration is configured.
    pub fn hw_device_ctx(&self) -> Option<*mut AVBufferRef> {
        let ctx = self.decoder.as_deref_except().hw_device_ctx;
        if ctx.is_null() { None } else { Some(ctx) }
    }

    /// Sends a packet to the decoder.
    pub fn send_packet(&mut self, packet: &Packet) -> Result<(), FfmpegError> {
        // Safety: `packet` is a valid pointer, and `self.decoder` is a valid pointer.
        FfmpegErrorCode(unsafe { avcodec_send_packet(self.decoder.as_mut_ptr(), packet.as_ptr()) }).result()?;
        Ok(())
    }

    /// Sends an end-of-file packet to the decoder.
    pub fn send_eof(&mut self) -> Result<(), FfmpegError> {
        // Safety: `self.decoder` is a valid pointer.
        FfmpegErrorCode(unsafe { avcodec_send_packet(self.decoder.as_mut_ptr(), std::ptr::null()) }).result()?;
        Ok(())
    }

    /// Flushes the decoder, draining all buffered frames and collecting them
    /// into a [`Vec`].
    ///
    /// This sends an EOF signal, receives every remaining frame, then resets
    /// the decoder so that new packets can be sent afterwards.  Use this when
    /// you need to force-drain buffered frames mid-stream (e.g. on a seek or
    /// when you need all frames *right now*).
    pub fn flush(&mut self) -> Result<Vec<GenericFrame>, FfmpegError> {
        self.send_eof()?;

        let mut frames = Vec::new();
        while let Some(frame) = self.receive_frame()? {
            frames.push(frame);
        }

        // Reset internal codec state so new packets can be accepted.
        // Safety: `self.decoder` is a valid, opened codec context.
        unsafe { avcodec_flush_buffers(self.decoder.as_mut_ptr()) };

        Ok(frames)
    }

    /// Receives a frame from the decoder.
    pub fn receive_frame(&mut self) -> Result<Option<GenericFrame>, FfmpegError> {
        let mut frame = GenericFrame::new()?;

        // Safety: `frame` is a valid pointer, and `self.decoder` is a valid pointer.
        let ret = FfmpegErrorCode(unsafe { avcodec_receive_frame(self.decoder.as_mut_ptr(), frame.as_mut_ptr()) });

        match ret {
            FfmpegErrorCode::Eagain | FfmpegErrorCode::Eof => Ok(None),
            code if code.is_success() => {
                frame.set_time_base(self.decoder.as_deref_except().time_base);
                Ok(Some(frame))
            }
            code => Err(FfmpegError::Code(code)),
        }
    }
}

impl VideoDecoder {
    /// Returns the width of the video frame.
    pub const fn width(&self) -> i32 {
        self.0.decoder.as_deref_except().width
    }

    /// Returns the height of the video frame.
    pub const fn height(&self) -> i32 {
        self.0.decoder.as_deref_except().height
    }

    /// Returns the pixel format of the video frame.
    pub const fn pixel_format(&self) -> AVPixelFormat {
        AVPixelFormat(self.0.decoder.as_deref_except().pix_fmt)
    }

    /// Returns the frame rate of the video frame.
    pub fn frame_rate(&self) -> Rational {
        self.0.decoder.as_deref_except().framerate.into()
    }

    /// Returns the sample aspect ratio of the video frame.
    pub fn sample_aspect_ratio(&self) -> Rational {
        self.0.decoder.as_deref_except().sample_aspect_ratio.into()
    }

    /// Receives a frame from the decoder.
    pub fn receive_frame(&mut self) -> Result<Option<VideoFrame>, FfmpegError> {
        Ok(self.0.receive_frame()?.map(|frame| frame.video()))
    }

    /// Flushes decoder and forces it to return frames.
    pub fn flush(&mut self) -> Result<Vec<VideoFrame>, FfmpegError> {
        Ok(self.0.flush()?.into_iter().map(|frame| frame.video()).collect())
    }
}

impl std::ops::Deref for VideoDecoder {
    type Target = GenericDecoder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for VideoDecoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AudioDecoder {
    /// Returns the sample rate of the audio frame.
    pub const fn sample_rate(&self) -> i32 {
        self.0.decoder.as_deref_except().sample_rate
    }

    /// Returns the number of channels in the audio frame.
    pub const fn channels(&self) -> i32 {
        self.0.decoder.as_deref_except().ch_layout.nb_channels
    }

    /// Returns the sample format of the audio frame.
    pub const fn sample_format(&self) -> AVSampleFormat {
        AVSampleFormat(self.0.decoder.as_deref_except().sample_fmt)
    }

    /// Receives a frame from the decoder.
    pub fn receive_frame(&mut self) -> Result<Option<AudioFrame>, FfmpegError> {
        Ok(self.0.receive_frame()?.map(|frame| frame.audio()))
    }
}

impl std::ops::Deref for AudioDecoder {
    type Target = GenericDecoder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AudioDecoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::num::NonZero;

    use crate::codec::DecoderCodec;
    use crate::decoder::{Decoder, DecoderOptions};
    use crate::io::Input;
    use crate::{AVCodecID, AVMediaType};

    #[test]
    fn test_generic_decoder_debug() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let stream = streams
            .iter()
            .find(|s| {
                s.codec_parameters()
                    .map(|p| AVMediaType(p.codec_type) == AVMediaType::Video)
                    .unwrap_or(false)
            })
            .expect("No video stream found");
        let codec_params = stream.codec_parameters().expect("Missing codec parameters");
        assert_eq!(
            AVMediaType(codec_params.codec_type),
            AVMediaType::Video,
            "Expected the stream to be a video stream"
        );
        let decoder_options = DecoderOptions {
            codec: Some(DecoderCodec::new(AVCodecID::H264).expect("Failed to find H264 codec")),
            thread_count: 2,
            hardware_acceleration: None,
            ..Default::default()
        };
        let decoder = Decoder::with_options(&stream, decoder_options).expect("Failed to create Decoder");
        let generic_decoder = match decoder {
            Decoder::Video(video_decoder) => video_decoder.0,
            Decoder::Audio(audio_decoder) => audio_decoder.0,
        };

        insta::assert_debug_snapshot!(generic_decoder, @r"
        Decoder {
            time_base: Some(
                Rational {
                    numerator: 1,
                    denominator: 15360,
                },
            ),
            codec_type: AVMediaType::Video,
        }
        ");
    }

    #[test]
    fn test_video_decoder_debug() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let stream = streams
            .iter()
            .find(|s| {
                s.codec_parameters()
                    .map(|p| AVMediaType(p.codec_type) == AVMediaType::Video)
                    .unwrap_or(false)
            })
            .expect("No video stream found");
        let codec_params = stream.codec_parameters().expect("Missing codec parameters");
        assert_eq!(
            AVMediaType(codec_params.codec_type),
            AVMediaType::Video,
            "Expected the stream to be a video stream"
        );

        let decoder_options = DecoderOptions {
            codec: Some(DecoderCodec::new(AVCodecID::H264).expect("Failed to find H264 codec")),
            thread_count: 2,
            hardware_acceleration: None,
            ..Default::default()
        };
        let decoder = Decoder::with_options(&stream, decoder_options).expect("Failed to create Decoder");

        let generic_decoder = match decoder {
            Decoder::Video(video_decoder) => video_decoder,
            _ => panic!("Expected a video decoder, got something else"),
        };

        insta::assert_debug_snapshot!(generic_decoder, @r"
        VideoDecoder {
            time_base: Some(
                Rational {
                    numerator: 1,
                    denominator: 15360,
                },
            ),
            width: 3840,
            height: 2160,
            pixel_format: AVPixelFormat::Yuv420p,
            frame_rate: Rational {
                numerator: 60,
                denominator: 1,
            },
            sample_aspect_ratio: Rational {
                numerator: 1,
                denominator: 1,
            },
        }
        ");
    }

    #[test]
    fn test_audio_decoder_debug() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let stream = streams
            .iter()
            .find(|s| {
                s.codec_parameters()
                    .map(|p| AVMediaType(p.codec_type) == AVMediaType::Audio)
                    .unwrap_or(false)
            })
            .expect("No audio stream found");
        let codec_params = stream.codec_parameters().expect("Missing codec parameters");
        assert_eq!(
            AVMediaType(codec_params.codec_type),
            AVMediaType::Audio,
            "Expected the stream to be an audio stream"
        );
        let decoder_options = DecoderOptions {
            codec: Some(DecoderCodec::new(AVCodecID::Aac).expect("Failed to find AAC codec")),
            thread_count: 2,
            hardware_acceleration: None,
            ..Default::default()
        };
        let decoder = Decoder::with_options(&stream, decoder_options).expect("Failed to create Decoder");
        let audio_decoder = match decoder {
            Decoder::Audio(audio_decoder) => audio_decoder,
            _ => panic!("Expected an audio decoder, got something else"),
        };

        insta::assert_debug_snapshot!(audio_decoder, @r"
        AudioDecoder {
            time_base: Some(
                Rational {
                    numerator: 1,
                    denominator: 48000,
                },
            ),
            sample_rate: 48000,
            channels: 2,
            sample_fmt: AVSampleFormat::Fltp,
        }
        ");
    }

    #[test]
    fn test_decoder_options_default() {
        let default_options = DecoderOptions::default();

        assert!(default_options.codec.is_none(), "Expected default codec to be None");
        assert_eq!(default_options.thread_count, 1, "Expected default thread_count to be 1");
    }

    #[test]
    fn test_decoder_new() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let stream = streams
            .iter()
            .find(|s| {
                s.codec_parameters()
                    .map(|p| AVMediaType(p.codec_type) == AVMediaType::Video)
                    .unwrap_or(false)
            })
            .expect("No video stream found");

        let decoder_result = Decoder::new(&stream);
        assert!(decoder_result.is_ok(), "Expected Decoder::new to succeed, but it failed");

        let decoder = decoder_result.unwrap();
        if let Decoder::Video(video_decoder) = decoder {
            assert_eq!(video_decoder.width(), 3840, "Expected valid width for video stream");
            assert_eq!(video_decoder.height(), 2160, "Expected valid height for video stream");
        } else {
            panic!("Expected a video decoder, but got a different type");
        }
    }

    #[test]
    fn test_decoder_with_options_missing_codec_parameters() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let mut input = Input::open(valid_file_path).expect("Failed to open valid file");
        let mut streams = input.streams_mut();
        let mut stream = streams.get(0).expect("Expected a valid stream");
        // Safety: Stream is a valid pointer.
        let codecpar = unsafe { (*stream.as_mut_ptr()).codecpar };
        // Safety: We are setting the `codecpar` to `null` to simulate a missing codec parameters.
        unsafe {
            (*stream.as_mut_ptr()).codecpar = std::ptr::null_mut();
        }
        let decoder_result = Decoder::with_options(&stream, DecoderOptions::default());
        // Safety: Stream is a valid pointer.
        unsafe {
            (*stream.as_mut_ptr()).codecpar = codecpar;
        }

        assert!(decoder_result.is_err(), "Expected Decoder creation to fail");
        if let Err(err) = decoder_result {
            match err {
                crate::error::FfmpegError::NoDecoder => (),
                _ => panic!("Unexpected error type: {err:?}"),
            }
        }
    }

    #[test]
    fn test_decoder_with_options_non_video_audio_codec_type() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let mut input = Input::open(valid_file_path).expect("Failed to open valid file");
        let mut streams = input.streams_mut();
        let mut stream = streams.get(0).expect("Expected a valid stream");
        // Safety: We are setting the `codecpar` to `null` to simulate a missing codec parameters.
        let codecpar = unsafe { (*stream.as_mut_ptr()).codecpar };
        // Safety: We are setting the `codec_type` to `AVMEDIA_TYPE_SUBTITLE` to simulate a non-video/audio codec type.
        unsafe {
            (*codecpar).codec_type = AVMediaType::Subtitle.into();
        }
        let decoder_result = Decoder::with_options(&stream, DecoderOptions::default());

        assert!(
            decoder_result.is_err(),
            "Expected Decoder creation to fail for non-video/audio codec type"
        );
        if let Err(err) = decoder_result {
            match err {
                crate::error::FfmpegError::NoDecoder => (),
                _ => panic!("Unexpected error type: {err:?}"),
            }
        }
    }

    #[test]
    fn test_video_decoder_deref_mut_safe() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let stream = streams
            .iter()
            .find(|s| {
                s.codec_parameters()
                    .map(|p| AVMediaType(p.codec_type) == AVMediaType::Video)
                    .unwrap_or(false)
            })
            .expect("No video stream found");
        let decoder_options = DecoderOptions {
            codec: None,
            thread_count: 2,
            hardware_acceleration: None,
            ..Default::default()
        };
        let decoder = Decoder::with_options(&stream, decoder_options).expect("Failed to create Decoder");
        let mut video_decoder = match decoder {
            Decoder::Video(video_decoder) => video_decoder,
            _ => panic!("Expected a VideoDecoder, got something else"),
        };
        {
            let generic_decoder = &mut *video_decoder;
            let mut time_base = generic_decoder.time_base().expect("Failed to get time base of decoder");
            time_base.numerator = 1000;
            time_base.denominator = NonZero::new(1).unwrap();
            generic_decoder.decoder.as_deref_mut_except().time_base = time_base.into();
        }
        let generic_decoder = &*video_decoder;
        let time_base = generic_decoder.decoder.as_deref_except().time_base;

        assert_eq!(time_base.num, 1000, "Expected time_base.num to be updated via DerefMut");
        assert_eq!(time_base.den, 1, "Expected time_base.den to be updated via DerefMut");
    }

    #[test]
    fn test_audio_decoder_deref_mut() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let stream = streams
            .iter()
            .find(|s| {
                s.codec_parameters()
                    .map(|p| AVMediaType(p.codec_type) == AVMediaType::Audio)
                    .unwrap_or(false)
            })
            .expect("No audio stream found");
        let decoder_options = DecoderOptions {
            codec: None,
            thread_count: 2,
            hardware_acceleration: None,
            ..Default::default()
        };
        let decoder = Decoder::with_options(&stream, decoder_options).expect("Failed to create Decoder");
        let mut audio_decoder = match decoder {
            Decoder::Audio(audio_decoder) => audio_decoder,
            _ => panic!("Expected an AudioDecoder, got something else"),
        };
        {
            let generic_decoder = &mut *audio_decoder;
            let mut time_base = generic_decoder.time_base().expect("Failed to get time base of decoder");
            time_base.numerator = 48000;
            time_base.denominator = NonZero::new(1).unwrap();
            generic_decoder.decoder.as_deref_mut_except().time_base = time_base.into();
        }
        let generic_decoder = &*audio_decoder;
        let time_base = generic_decoder.decoder.as_deref_except().time_base;

        assert_eq!(time_base.num, 48000, "Expected time_base.num to be updated via DerefMut");
        assert_eq!(time_base.den, 1, "Expected time_base.den to be updated via DerefMut");
    }

    #[test]
    fn test_decoder_video() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let mut input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("No video stream found");
        let audio_stream = streams.best(AVMediaType::Audio).expect("No audio stream found");
        let mut video_decoder = Decoder::new(&video_stream)
            .expect("Failed to create decoder")
            .video()
            .expect("Failed to get video decoder");
        let mut audio_decoder = Decoder::new(&audio_stream)
            .expect("Failed to create decoder")
            .audio()
            .expect("Failed to get audio decoder");
        let mut video_frames = Vec::new();
        let mut audio_frames = Vec::new();

        let video_stream_index = video_stream.index();
        let audio_stream_index = audio_stream.index();

        while let Some(packet) = input.receive_packet().expect("Failed to receive packet") {
            if packet.stream_index() == video_stream_index {
                video_decoder.send_packet(&packet).expect("Failed to send packet");
                while let Some(frame) = video_decoder.receive_frame().expect("Failed to receive frame") {
                    video_frames.push(frame);
                }
            } else if packet.stream_index() == audio_stream_index {
                audio_decoder.send_packet(&packet).expect("Failed to send packet");
                while let Some(frame) = audio_decoder.receive_frame().expect("Failed to receive frame") {
                    audio_frames.push(frame);
                }
            }
        }

        video_decoder.send_eof().expect("Failed to send eof");
        while let Some(frame) = video_decoder.receive_frame().expect("Failed to receive frame") {
            video_frames.push(frame);
        }

        audio_decoder.send_eof().expect("Failed to send eof");
        while let Some(frame) = audio_decoder.receive_frame().expect("Failed to receive frame") {
            audio_frames.push(frame);
        }

        insta::assert_debug_snapshot!("test_decoder_video", video_frames);
        insta::assert_debug_snapshot!("test_decoder_audio", audio_frames);
    }

    /// Helper: creates a video decoder with the given options for the test asset,
    /// sends packets one at a time and records how many packets were sent before
    /// each frame was received. Returns the list of "packets sent so far" values,
    /// one entry per decoded frame (in decode-output order).
    fn packets_before_frames(options: DecoderOptions) -> Vec<usize> {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let mut input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("No video stream found");
        let video_stream_index = video_stream.index();

        let mut decoder = Decoder::with_options(&video_stream, options)
            .expect("Failed to create Decoder")
            .video()
            .expect("Failed to get video decoder");

        let mut result = Vec::new();
        let mut packets_sent: usize = 0;

        while let Some(packet) = input.receive_packet().expect("Failed to receive packet") {
            if packet.stream_index() != video_stream_index {
                continue;
            }
            decoder.send_packet(&packet).expect("Failed to send packet");
            packets_sent += 1;

            while let Some(_frame) = decoder.receive_frame().expect("Failed to receive frame") {
                result.push(packets_sent);
            }
        }

        // Flush
        decoder.send_eof().expect("Failed to send eof");
        while let Some(_frame) = decoder.receive_frame().expect("Failed to receive frame") {
            result.push(packets_sent);
        }

        result
    }

    #[test]
    fn test_frame_threading_adds_latency() {
        use crate::decoder::ThreadType;

        // Default-ish config with frame threading and multiple threads:
        // the decoder must buffer thread_count-1 extra frames from threading
        // plus has_b_frames frames from H264 reordering.
        let buffered = packets_before_frames(DecoderOptions {
            codec: Some(DecoderCodec::new(AVCodecID::H264).expect("codec")),
            thread_count: 4,
            thread_type: ThreadType::Frame,
            low_delay: false,
            hardware_acceleration: None,
        });

        // The first frame should not come out until several packets have been
        // sent (thread_count-1 = 3 from frame threading + has_b_frames = 2
        // from H264 reorder = at least 5 packets before first output).
        assert!(
            buffered[0] >= 4,
            "With frame threading (4 threads) the first frame should require >= 4 packets, got {}",
            buffered[0],
        );
    }

    #[test]
    fn test_low_delay_slice_threading_reduces_latency() {
        use crate::decoder::ThreadType;

        // Low-delay + slice-only threading: no frame-threading pipeline delay,
        // only the H264 reorder buffer (has_b_frames from the stream content).
        let low_delay = packets_before_frames(DecoderOptions {
            codec: Some(DecoderCodec::new(AVCodecID::H264).expect("codec")),
            thread_count: 4,
            thread_type: ThreadType::Slice,
            low_delay: true,
            hardware_acceleration: None,
        });

        // Also measure the default frame-threading config for comparison.
        let frame_threaded = packets_before_frames(DecoderOptions {
            codec: Some(DecoderCodec::new(AVCodecID::H264).expect("codec")),
            thread_count: 4,
            thread_type: ThreadType::Frame,
            low_delay: false,
            hardware_acceleration: None,
        });

        // Low-delay should produce the first frame strictly sooner than
        // frame threading (which adds thread_count-1 extra frames of delay).
        assert!(
            low_delay[0] < frame_threaded[0],
            "Low-delay first frame after {} packets, frame-threaded after {} — expected low_delay < frame_threaded",
            low_delay[0],
            frame_threaded[0],
        );

        // Both should decode the same total number of frames.
        assert_eq!(
            low_delay.len(),
            frame_threaded.len(),
            "Both configurations should produce the same number of frames",
        );
    }

    #[test]
    fn test_flush_forces_buffered_frames_out() {
        let valid_file_path = "../../assets/avc_aac_large.mp4";
        let mut input = Input::open(valid_file_path).expect("Failed to open valid file");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("No video stream found");
        let video_stream_index = video_stream.index();

        // Use default (frame-threaded) config so the decoder actually buffers.
        let mut decoder = Decoder::with_options(
            &video_stream,
            DecoderOptions {
                codec: Some(DecoderCodec::new(AVCodecID::H264).expect("codec")),
                thread_count: 4,
                hardware_acceleration: None,
                ..Default::default()
            },
        )
        .expect("Failed to create Decoder")
        .video()
        .expect("Failed to get video decoder");

        // Send a handful of packets — with frame threading + B-frames the
        // decoder will be buffering internally and receive_frame returns None.
        let mut packets_sent = 0usize;
        let mut frames_before_flush = 0usize;
        while let Some(packet) = input.receive_packet().expect("receive_packet") {
            if packet.stream_index() != video_stream_index {
                continue;
            }
            decoder.send_packet(&packet).expect("send_packet");
            packets_sent += 1;

            while let Some(_) = decoder.receive_frame().expect("receive_frame") {
                frames_before_flush += 1;
            }

            if packets_sent >= 5 {
                break;
            }
        }

        // Flush: this should force all buffered frames out.
        let flushed = decoder.flush().expect("flush");
        assert!(
            !flushed.is_empty(),
            "flush() should have returned buffered frames, but got none \
             (packets_sent={packets_sent}, frames_before_flush={frames_before_flush})",
        );

        let total_after_flush = frames_before_flush + flushed.len();
        assert!(
            total_after_flush > frames_before_flush,
            "flush() must produce additional frames beyond what receive_frame already returned",
        );

        // After flush the decoder should accept new packets and keep working.
        let mut frames_after_flush = 0usize;
        while let Some(packet) = input.receive_packet().expect("receive_packet") {
            if packet.stream_index() != video_stream_index {
                continue;
            }
            decoder.send_packet(&packet).expect("send_packet after flush");
            while let Some(_) = decoder.receive_frame().expect("receive_frame after flush") {
                frames_after_flush += 1;
            }
        }

        // Final drain.
        decoder.send_eof().expect("send_eof");
        while let Some(_) = decoder.receive_frame().expect("receive_frame final") {
            frames_after_flush += 1;
        }

        assert!(
            frames_after_flush > 0,
            "Decoder should produce frames after flush() + new packets",
        );
    }
}
