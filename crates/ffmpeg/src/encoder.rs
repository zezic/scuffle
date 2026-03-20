use std::ptr::NonNull;

use crate::codec::EncoderCodec;
use crate::dict::Dictionary;
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::frame::{AudioChannelLayout, GenericFrame};
use crate::io::Output;
use crate::packet::Packet;
use crate::rational::Rational;
use crate::smart_object::SmartPtr;
use crate::{AVFormatFlags, AVPixelFormat, AVSampleFormat};

/// Wrapper around a raw `*mut AVBufferRef` for hardware device context.
/// Implements Default (null pointer) so bon::Builder can use `#[builder(default)]`.
#[derive(Clone, Copy)]
pub struct HwDeviceCtxPtr(pub *mut AVBufferRef);

impl Default for HwDeviceCtxPtr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

impl HwDeviceCtxPtr {
    /// Creates a new `HwDeviceCtxPtr` from a raw pointer.
    pub fn new(ptr: *mut AVBufferRef) -> Self {
        Self(ptr)
    }

    /// Returns `true` if the pointer is null.
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

/// Represents an encoder.
pub struct Encoder {
    incoming_time_base: Rational,
    outgoing_time_base: Rational,
    encoder: SmartPtr<AVCodecContext>,
    stream_index: i32,
    previous_dts: i64,
}

/// Safety: `Encoder` can be sent between threads.
unsafe impl Send for Encoder {}

/// Represents the settings for a video encoder.
#[derive(bon::Builder)]
pub struct VideoEncoderSettings {
    width: i32,
    height: i32,
    frame_rate: Rational,
    pixel_format: AVPixelFormat,
    gop_size: Option<i32>,
    qmax: Option<i32>,
    qmin: Option<i32>,
    thread_count: Option<i32>,
    thread_type: Option<i32>,
    sample_aspect_ratio: Option<Rational>,
    bitrate: Option<i64>,
    rc_min_rate: Option<i64>,
    rc_max_rate: Option<i64>,
    rc_buffer_size: Option<i32>,
    max_b_frames: Option<i32>,
    codec_specific_options: Option<Dictionary>,
    flags: Option<i32>,
    flags2: Option<i32>,
    /// Force low-delay mode (`AV_CODEC_FLAG_LOW_DELAY`).
    ///
    /// For x264, combine this with `codec_specific_options` containing
    /// `tune=zerolatency` and `max_b_frames(0)` to get truly non-buffering
    /// encoding (one packet out per frame in).
    low_delay: Option<bool>,
    /// Color range: 1 = MPEG/TV (16-235), 2 = JPEG/Full (0-255).
    color_range: Option<u32>,
}

impl VideoEncoderSettings {
    fn apply(self, encoder: &mut AVCodecContext) -> Result<(), FfmpegError> {
        if self.width <= 0 || self.height <= 0 || self.frame_rate.numerator <= 0 || self.pixel_format == AVPixelFormat::None
        {
            return Err(FfmpegError::Arguments(
                "width, height, frame_rate and pixel_format must be set",
            ));
        }

        encoder.width = self.width;
        encoder.height = self.height;
        encoder.pix_fmt = self.pixel_format.into();
        encoder.sample_aspect_ratio = self
            .sample_aspect_ratio
            .map(Into::into)
            .unwrap_or(encoder.sample_aspect_ratio);
        encoder.framerate = self.frame_rate.into();
        encoder.thread_count = self.thread_count.unwrap_or(encoder.thread_count);
        encoder.thread_type = self.thread_type.unwrap_or(encoder.thread_type);
        encoder.gop_size = self.gop_size.unwrap_or(encoder.gop_size);
        encoder.qmax = self.qmax.unwrap_or(encoder.qmax);
        encoder.qmin = self.qmin.unwrap_or(encoder.qmin);
        encoder.bit_rate = self.bitrate.unwrap_or(encoder.bit_rate);
        encoder.rc_min_rate = self.rc_min_rate.unwrap_or(encoder.rc_min_rate);
        encoder.rc_max_rate = self.rc_max_rate.unwrap_or(encoder.rc_max_rate);
        encoder.rc_buffer_size = self.rc_buffer_size.unwrap_or(encoder.rc_buffer_size);
        encoder.max_b_frames = self.max_b_frames.unwrap_or(encoder.max_b_frames);
        encoder.flags = self.flags.unwrap_or(encoder.flags);
        encoder.flags2 = self.flags2.unwrap_or(encoder.flags2);
        if self.low_delay.unwrap_or(false) {
            encoder.flags |= AV_CODEC_FLAG_LOW_DELAY as i32;
        }
        if let Some(cr) = self.color_range {
            encoder.color_range = cr;
        }

        Ok(())
    }
}

/// Represents the settings for an audio encoder.
#[derive(bon::Builder)]
pub struct AudioEncoderSettings {
    sample_rate: i32,
    ch_layout: AudioChannelLayout,
    sample_fmt: AVSampleFormat,
    thread_count: Option<i32>,
    thread_type: Option<i32>,
    bitrate: Option<i64>,
    rc_min_rate: Option<i64>,
    rc_max_rate: Option<i64>,
    rc_buffer_size: Option<i32>,
    codec_specific_options: Option<Dictionary>,
    flags: Option<i32>,
    flags2: Option<i32>,
}

impl AudioEncoderSettings {
    fn apply(self, encoder: &mut AVCodecContext) -> Result<(), FfmpegError> {
        if self.sample_rate <= 0 || self.sample_fmt == AVSampleFormat::None {
            return Err(FfmpegError::Arguments(
                "sample_rate, channel_layout and sample_fmt must be set",
            ));
        }

        encoder.sample_rate = self.sample_rate;
        self.ch_layout.apply(&mut encoder.ch_layout);
        encoder.sample_fmt = self.sample_fmt.into();
        encoder.thread_count = self.thread_count.unwrap_or(encoder.thread_count);
        encoder.thread_type = self.thread_type.unwrap_or(encoder.thread_type);
        encoder.bit_rate = self.bitrate.unwrap_or(encoder.bit_rate);
        encoder.rc_min_rate = self.rc_min_rate.unwrap_or(encoder.rc_min_rate);
        encoder.rc_max_rate = self.rc_max_rate.unwrap_or(encoder.rc_max_rate);
        encoder.rc_buffer_size = self.rc_buffer_size.unwrap_or(encoder.rc_buffer_size);
        encoder.flags = self.flags.unwrap_or(encoder.flags);
        encoder.flags2 = self.flags2.unwrap_or(encoder.flags2);

        Ok(())
    }
}

/// Represents the settings for an encoder.
pub enum EncoderSettings {
    /// Video encoder settings.
    Video(VideoEncoderSettings),
    /// Audio encoder settings.
    Audio(AudioEncoderSettings),
}

impl EncoderSettings {
    fn apply(self, encoder: &mut AVCodecContext) -> Result<(), FfmpegError> {
        match self {
            EncoderSettings::Video(video_settings) => video_settings.apply(encoder),
            EncoderSettings::Audio(audio_settings) => audio_settings.apply(encoder),
        }
    }

    const fn codec_specific_options(&mut self) -> Option<&mut Dictionary> {
        match self {
            EncoderSettings::Video(video_settings) => video_settings.codec_specific_options.as_mut(),
            EncoderSettings::Audio(audio_settings) => audio_settings.codec_specific_options.as_mut(),
        }
    }
}

impl From<VideoEncoderSettings> for EncoderSettings {
    fn from(settings: VideoEncoderSettings) -> Self {
        EncoderSettings::Video(settings)
    }
}

impl From<AudioEncoderSettings> for EncoderSettings {
    fn from(settings: AudioEncoderSettings) -> Self {
        EncoderSettings::Audio(settings)
    }
}

impl Encoder {
    /// Creates a new encoder.
    pub fn new<T: Send + Sync>(
        codec: EncoderCodec,
        output: &mut Output<T>,
        incoming_time_base: impl Into<Rational>,
        outgoing_time_base: impl Into<Rational>,
        settings: impl Into<EncoderSettings>,
    ) -> Result<Self, FfmpegError> {
        Self::new_inner(codec, output, incoming_time_base, outgoing_time_base, settings, None, None)
    }

    /// Creates a new encoder with a hardware device context.
    /// The `hw_device_ctx` is assigned before `avcodec_open2` is called.
    pub fn new_with_hw_device<T: Send + Sync>(
        codec: EncoderCodec,
        output: &mut Output<T>,
        incoming_time_base: impl Into<Rational>,
        outgoing_time_base: impl Into<Rational>,
        settings: impl Into<EncoderSettings>,
        hw_device_ctx: *mut AVBufferRef,
    ) -> Result<Self, FfmpegError> {
        Self::new_inner(codec, output, incoming_time_base, outgoing_time_base, settings, Some(hw_device_ctx), None)
    }

    /// Creates a new encoder with both hardware device and frames contexts.
    /// Required by encoders like h264_nvenc that need `hw_frames_ctx` to know
    /// the GPU frame pool layout when receiving GPU frames directly.
    pub fn new_with_hw_contexts<T: Send + Sync>(
        codec: EncoderCodec,
        output: &mut Output<T>,
        incoming_time_base: impl Into<Rational>,
        outgoing_time_base: impl Into<Rational>,
        settings: impl Into<EncoderSettings>,
        hw_device_ctx: *mut AVBufferRef,
        hw_frames_ctx: *mut AVBufferRef,
    ) -> Result<Self, FfmpegError> {
        Self::new_inner(codec, output, incoming_time_base, outgoing_time_base, settings, Some(hw_device_ctx), Some(hw_frames_ctx))
    }

    fn new_inner<T: Send + Sync>(
        codec: EncoderCodec,
        output: &mut Output<T>,
        incoming_time_base: impl Into<Rational>,
        outgoing_time_base: impl Into<Rational>,
        settings: impl Into<EncoderSettings>,
        hw_device_ctx: Option<*mut AVBufferRef>,
        hw_frames_ctx: Option<*mut AVBufferRef>,
    ) -> Result<Self, FfmpegError> {
        if codec.as_ptr().is_null() {
            return Err(FfmpegError::NoEncoder);
        }

        let mut settings = settings.into();

        let global_header = output
            .output_flags()
            .is_some_and(|flags| flags & AVFormatFlags::GlobalHeader != 0);

        let destructor = |ptr: &mut *mut AVCodecContext| {
            // Safety: `avcodec_free_context` is safe to call when the pointer is valid, and it is because it comes from `avcodec_alloc_context3`.
            unsafe { avcodec_free_context(ptr) };
        };

        // Safety: `avcodec_alloc_context3` is safe to call.
        let encoder = unsafe { avcodec_alloc_context3(codec.as_ptr()) };

        // Safety: The pointer here is valid and the destructor has been setup to handle the cleanup.
        let mut encoder = unsafe { SmartPtr::wrap_non_null(encoder, destructor) }.ok_or(FfmpegError::Alloc)?;

        let mut ost = output.add_stream(None).ok_or(FfmpegError::NoStream)?;

        let encoder_mut = encoder.as_deref_mut_except();

        let incoming_time_base = incoming_time_base.into();
        let outgoing_time_base = outgoing_time_base.into();

        encoder_mut.time_base = incoming_time_base.into();

        let mut codec_options = settings.codec_specific_options().cloned();

        let codec_options_ptr = codec_options
            .as_mut()
            .map(|options| options.as_mut_ptr_ref() as *mut *mut _)
            .unwrap_or(std::ptr::null_mut());

        settings.apply(encoder_mut)?;

        // Set hardware device context before avcodec_open2 if provided
        if let Some(hw_ctx) = hw_device_ctx {
            // Safety: av_buffer_ref creates a new reference to the buffer
            encoder_mut.hw_device_ctx = unsafe { av_buffer_ref(hw_ctx) };
        }

        // Set hardware frames context before avcodec_open2 if provided.
        // Required by nvenc when receiving GPU frames (CUDA pixel format).
        if let Some(hw_frames) = hw_frames_ctx {
            encoder_mut.hw_frames_ctx = unsafe { av_buffer_ref(hw_frames) };
        }

        if global_header {
            encoder_mut.flags |= AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }

        // Safety: `avcodec_open2` is safe to call, 'encoder' and 'codec' and
        // 'codec_options_ptr' are a valid pointers.
        FfmpegErrorCode(unsafe { avcodec_open2(encoder_mut, codec.as_ptr(), codec_options_ptr) }).result()?;

        // For VideoToolbox encoder: set MaxFrameDelayCount=0 to force synchronous output.
        // Without this, VT buffers frames indefinitely (kVTUnlimitedFrameDelayCount=-1),
        // requiring flush+recreate at every segment boundary.
        // VTEncContext layout (ffmpeg 8.x): AVClass*(8) + codec_id(4) + pad(4) + session(8)
        #[cfg(target_os = "macos")]
        {
            // Only apply to VideoToolbox encoders — check codec name
            let codec_name = if !encoder_mut.codec.is_null() {
                let name_ptr = unsafe { (*encoder_mut.codec).name };
                if !name_ptr.is_null() {
                    unsafe { std::ffi::CStr::from_ptr(name_ptr) }.to_str().unwrap_or("")
                } else { "" }
            } else { "" };
            if codec_name.contains("videotoolbox") {
            if !encoder_mut.priv_data.is_null() {
                // session is the 3rd field in VTEncContext, at offset 16
                let session_ptr = unsafe {
                    *(encoder_mut.priv_data.cast::<u8>().add(16) as *const *const std::ffi::c_void)
                };
                if !session_ptr.is_null() {
                    #[link(name = "VideoToolbox", kind = "framework")]
                    #[link(name = "CoreFoundation", kind = "framework")]
                    unsafe extern "C" {
                        fn VTSessionSetProperty(
                            session: *const std::ffi::c_void,
                            key: *const std::ffi::c_void,
                            value: *const std::ffi::c_void,
                        ) -> i32;
                        fn CFNumberCreate(
                            allocator: *const std::ffi::c_void,
                            the_type: isize,
                            value_ptr: *const std::ffi::c_void,
                        ) -> *const std::ffi::c_void;
                        fn CFRelease(cf: *const std::ffi::c_void);
                        static kVTCompressionPropertyKey_MaxFrameDelayCount: *const std::ffi::c_void;
                    }
                    unsafe {
                        let zero: i32 = 0;
                        // kCFNumberSInt32Type = 3
                        let num = CFNumberCreate(
                            std::ptr::null(),
                            3,
                            &zero as *const i32 as *const std::ffi::c_void,
                        );
                        if !num.is_null() {
                            VTSessionSetProperty(
                                session_ptr,
                                kVTCompressionPropertyKey_MaxFrameDelayCount,
                                num,
                            );
                            CFRelease(num);
                        }
                    }
                }
            }
            }
        }

        // Use the post-open time_base for PTS conversion — encoders may change it during open.
        let incoming_time_base: Rational = encoder_mut.time_base.into();

        // Safety: The pointer here is valid.
        let ost_mut = unsafe { NonNull::new(ost.as_mut_ptr()).ok_or(FfmpegError::NoStream)?.as_mut() };

        // Safety: `avcodec_parameters_from_context` is safe to call, 'ost' and
        // 'encoder' are valid pointers.
        FfmpegErrorCode(unsafe { avcodec_parameters_from_context(ost_mut.codecpar, encoder_mut) }).result()?;

        ost.set_time_base(outgoing_time_base);

        Ok(Self {
            incoming_time_base,
            outgoing_time_base,
            encoder,
            stream_index: ost.index(),
            previous_dts: i64::MIN,
        })
    }

    /// Sends an EOF frame to the encoder.
    pub fn send_eof(&mut self) -> Result<(), FfmpegError> {
        // Safety: `self.encoder` is a valid pointer.
        FfmpegErrorCode(unsafe { avcodec_send_frame(self.encoder.as_mut_ptr(), std::ptr::null()) }).result()?;
        Ok(())
    }

    /// Reset the internal codec state / flush internal buffers.
    /// Should be called e.g. when seeking or when switching to a different stream.
    /// Check if the encoder supports flushing by calling `has_flush_capability`.
    pub fn flush_buffers(&mut self) -> Result<(), FfmpegError> {
        // Safety: `self.encoder` is a valid pointer.
        unsafe { avcodec_flush_buffers(self.encoder.as_mut_ptr()) };
        Ok(())
    }

    /// Check if the encoder supports flushing.
    pub fn has_flush_capability(&self) -> bool {
        // Safety: `self.encoder` is a valid pointer.
        unsafe {
            let ctx = self.encoder.as_deref_except();
            let codec = ctx.codec.as_ref().unwrap();
            codec.capabilities & AV_CODEC_CAP_ENCODER_FLUSH as i32 != 0
        }
    }

    /// Sends a frame to the encoder.
    pub fn send_frame(&mut self, frame: &GenericFrame) -> Result<(), FfmpegError> {
        // Safety: `self.encoder` and `frame` are valid pointers.
        FfmpegErrorCode(unsafe { avcodec_send_frame(self.encoder.as_mut_ptr(), frame.as_ptr()) }).result()?;
        Ok(())
    }

    /// Receives a packet from the encoder.
    pub fn receive_packet(&mut self) -> Result<Option<Packet>, FfmpegError> {
        let mut packet = Packet::new()?;

        // Safety: `self.encoder` and `packet` are valid pointers.
        let ret = FfmpegErrorCode(unsafe { avcodec_receive_packet(self.encoder.as_mut_ptr(), packet.as_mut_ptr()) });

        match ret {
            FfmpegErrorCode::Eagain | FfmpegErrorCode::Eof => Ok(None),
            code if code.is_success() => {
                if cfg!(debug_assertions) {
                    debug_assert!(
                        packet.dts().is_some(),
                        "packet dts is none, this should never happen, please report this bug"
                    );
                    let packet_dts = packet.dts().unwrap();
                    debug_assert!(
                        packet_dts >= self.previous_dts,
                        "packet dts is less than previous dts: {} >= {}",
                        packet_dts,
                        self.previous_dts
                    );
                    self.previous_dts = packet_dts;
                }

                packet.convert_timebase(self.incoming_time_base, self.outgoing_time_base);
                packet.set_stream_index(self.stream_index);
                Ok(Some(packet))
            }
            code => Err(FfmpegError::Code(code)),
        }
    }

    /// Returns the stream index of the encoder.
    pub const fn stream_index(&self) -> i32 {
        self.stream_index
    }

    /// Returns the incoming time base of the encoder.
    pub const fn incoming_time_base(&self) -> Rational {
        self.incoming_time_base
    }

    /// Returns the outgoing time base of the encoder.
    pub const fn outgoing_time_base(&self) -> Rational {
        self.outgoing_time_base
    }

    /// Returns the codec context of the encoder.
    pub fn codec_context(&self) -> *const AVCodecContext {
        self.encoder.as_ptr()
    }

    /// Audio only. The number of "priming" samples (padding) inserted by the
    /// encoder at the beginning of the audio. I.e. this number of leading
    /// decoded samples must be discarded by the caller to get the original audio
    /// without leading padding.
    ///
    /// Set by libavcodec. The timestamps on the output packets are
    /// adjusted by the encoder so that they always refer to the
    /// first sample of the data actually contained in the packet,
    /// including any added padding.  E.g. if the timebase is
    /// 1/samplerate and the timestamp of the first input sample is
    /// 0, the timestamp of the first output packet will be
    /// -initial_padding.
    pub fn initial_padding(&self) -> i32 {
        self.encoder.as_deref_except().initial_padding
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io::Write;

    use bytes::{Buf, Bytes};
    use rusty_ffmpeg::ffi::AVRational;
    use sha2::Digest;

    use crate::codec::EncoderCodec;
    use crate::decoder::Decoder;
    use crate::dict::Dictionary;
    use crate::encoder::{AudioChannelLayout, AudioEncoderSettings, Encoder, EncoderSettings, VideoEncoderSettings};
    use crate::error::FfmpegError;
    use crate::ffi::{AVCodecContext, AV_CODEC_FLAG_LOW_DELAY};
    use crate::io::{Input, Output, OutputOptions};
    use crate::rational::Rational;
    use crate::{AVChannelOrder, AVCodecID, AVMediaType, AVPixelFormat, AVSampleFormat};

    #[test]
    fn test_video_encoder_apply() {
        let width = 1920;
        let height = 1080;
        let frame_rate = 30;
        let pixel_format = AVPixelFormat::Yuv420p;
        let sample_aspect_ratio = 1;
        let gop_size = 12;
        let qmax = 31;
        let qmin = 1;
        let thread_count = 4;
        let thread_type = 2;
        let bitrate = 8_000;
        let rc_min_rate = 500_000;
        let rc_max_rate = 2_000_000;
        let rc_buffer_size = 1024;
        let max_b_frames = 3;
        let mut codec_specific_options = Dictionary::new();
        codec_specific_options.set("preset", "ultrafast").unwrap();
        codec_specific_options.set("crf", "23").unwrap();
        let flags = 0x01;
        let flags2 = 0x02;

        let low_delay = true;

        let settings = VideoEncoderSettings::builder()
            .width(width)
            .height(height)
            .frame_rate(frame_rate.into())
            .pixel_format(pixel_format)
            .sample_aspect_ratio(sample_aspect_ratio.into())
            .gop_size(gop_size)
            .qmax(qmax)
            .qmin(qmin)
            .thread_count(thread_count)
            .thread_type(thread_type)
            .bitrate(bitrate)
            .rc_min_rate(rc_min_rate)
            .rc_max_rate(rc_max_rate)
            .rc_buffer_size(rc_buffer_size)
            .max_b_frames(max_b_frames)
            .codec_specific_options(codec_specific_options)
            .flags(flags)
            .flags2(flags2)
            .low_delay(low_delay)
            .build();

        assert_eq!(settings.width, width);
        assert_eq!(settings.height, height);
        assert_eq!(settings.frame_rate, frame_rate.into());
        assert_eq!(settings.pixel_format, pixel_format);
        assert_eq!(settings.sample_aspect_ratio, Some(sample_aspect_ratio.into()));
        assert_eq!(settings.gop_size, Some(gop_size));
        assert_eq!(settings.qmax, Some(qmax));
        assert_eq!(settings.qmin, Some(qmin));
        assert_eq!(settings.thread_count, Some(thread_count));
        assert_eq!(settings.thread_type, Some(thread_type));
        assert_eq!(settings.bitrate, Some(bitrate));
        assert_eq!(settings.rc_min_rate, Some(rc_min_rate));
        assert_eq!(settings.rc_max_rate, Some(rc_max_rate));
        assert_eq!(settings.rc_buffer_size, Some(rc_buffer_size));
        assert_eq!(settings.max_b_frames, Some(max_b_frames));
        assert!(settings.codec_specific_options.is_some());
        let actual_codec_specific_options = settings.codec_specific_options.as_ref().unwrap();
        assert_eq!(actual_codec_specific_options.get(c"preset"), Some(c"ultrafast"));
        assert_eq!(actual_codec_specific_options.get(c"crf"), Some(c"23"));
        assert_eq!(settings.flags, Some(flags));
        assert_eq!(settings.flags2, Some(flags2));
        assert_eq!(settings.low_delay, Some(low_delay));

        // Safety: We are zeroing the memory for the encoder context.
        let mut encoder = unsafe { std::mem::zeroed::<AVCodecContext>() };
        let result = settings.apply(&mut encoder);
        assert!(result.is_ok(), "Failed to apply settings: {:?}", result.err());

        assert_eq!(encoder.width, width);
        assert_eq!(encoder.height, height);
        assert_eq!(AVPixelFormat(encoder.pix_fmt), pixel_format);
        assert_eq!(Rational::from(encoder.sample_aspect_ratio), sample_aspect_ratio.into());
        assert_eq!(Rational::from(encoder.framerate), frame_rate.into());
        assert_eq!(encoder.thread_count, thread_count);
        assert_eq!(encoder.thread_type, thread_type);
        assert_eq!(encoder.gop_size, gop_size);
        assert_eq!(encoder.qmax, qmax);
        assert_eq!(encoder.qmin, qmin);
        assert_eq!(encoder.bit_rate, bitrate);
        assert_eq!(encoder.rc_min_rate, rc_min_rate);
        assert_eq!(encoder.rc_max_rate, rc_max_rate);
        assert_eq!(encoder.rc_buffer_size, rc_buffer_size);
        assert_eq!(encoder.max_b_frames, max_b_frames);
        assert_eq!(encoder.flags, flags | AV_CODEC_FLAG_LOW_DELAY as i32);
        assert_eq!(encoder.flags2, flags2);
    }

    #[test]
    fn test_video_encoder_settings_apply_error() {
        let settings = VideoEncoderSettings::builder()
            .width(0)
            .height(0)
            .pixel_format(AVPixelFormat::Yuv420p)
            .frame_rate(0.into())
            .build();
        // Safety: We are zeroing the memory for the encoder context.
        let mut encoder = unsafe { std::mem::zeroed::<AVCodecContext>() };
        let result = settings.apply(&mut encoder);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            FfmpegError::Arguments("width, height, frame_rate and pixel_format must be set")
        );
    }

    #[test]
    fn test_audio_encoder_apply() {
        let sample_rate = 44100;
        let channel_count = 2;
        let sample_fmt = AVSampleFormat::S16;
        let thread_count = 4;
        let thread_type = 1;
        let bitrate = 128_000;
        let rc_min_rate = 64_000;
        let rc_max_rate = 256_000;
        let rc_buffer_size = 1024;
        let flags = 0x01;
        let flags2 = 0x02;

        let mut codec_specific_options = Dictionary::new();
        codec_specific_options
            .set(c"profile", c"high")
            .expect("Failed to set profile");

        let settings = AudioEncoderSettings::builder()
            .sample_rate(sample_rate)
            .ch_layout(AudioChannelLayout::new(channel_count).expect("channel_count is a valid value"))
            .sample_fmt(sample_fmt)
            .thread_count(thread_count)
            .thread_type(thread_type)
            .bitrate(bitrate)
            .rc_min_rate(rc_min_rate)
            .rc_max_rate(rc_max_rate)
            .rc_buffer_size(rc_buffer_size)
            .codec_specific_options(codec_specific_options)
            .flags(flags)
            .flags2(flags2)
            .build();

        assert_eq!(settings.sample_rate, sample_rate);
        assert_eq!(settings.ch_layout.channel_count(), 2);
        assert_eq!(settings.sample_fmt, sample_fmt);
        assert_eq!(settings.thread_count, Some(thread_count));
        assert_eq!(settings.thread_type, Some(thread_type));
        assert_eq!(settings.bitrate, Some(bitrate));
        assert_eq!(settings.rc_min_rate, Some(rc_min_rate));
        assert_eq!(settings.rc_max_rate, Some(rc_max_rate));
        assert_eq!(settings.rc_buffer_size, Some(rc_buffer_size));
        assert!(settings.codec_specific_options.is_some());

        let actual_codec_specific_options = settings.codec_specific_options.unwrap();
        assert_eq!(actual_codec_specific_options.get(c"profile"), Some(c"high"));

        assert_eq!(settings.flags, Some(flags));
        assert_eq!(settings.flags2, Some(flags2));
    }

    #[test]
    fn test_ch_layout_valid_layout() {
        // Safety: This is safe to call and the channel layout is allocated on the stack.
        let channel_layout = unsafe {
            AudioChannelLayout::wrap(crate::ffi::AVChannelLayout {
                order: AVChannelOrder::Native.into(),
                nb_channels: 2,
                u: crate::ffi::AVChannelLayout__bindgen_ty_1 { mask: 0b11 },
                opaque: std::ptr::null_mut(),
            })
        };

        channel_layout.validate().expect("channel_layout is a valid value");
    }

    #[test]
    fn test_ch_layout_invalid_layout() {
        // Safety: This is safe to call and the channel layout is allocated on the stack.
        let channel_layout = unsafe {
            AudioChannelLayout::wrap(crate::ffi::AVChannelLayout {
                order: AVChannelOrder::Unspecified.into(),
                nb_channels: 0,
                u: crate::ffi::AVChannelLayout__bindgen_ty_1 { mask: 0 },
                opaque: std::ptr::null_mut(),
            })
        };
        let result: Result<(), FfmpegError> = channel_layout.validate();
        assert_eq!(result.unwrap_err(), FfmpegError::Arguments("invalid channel layout"));
    }

    #[test]
    fn test_audio_encoder_settings_apply_error() {
        let settings = AudioEncoderSettings::builder()
            .sample_rate(0)
            .sample_fmt(AVSampleFormat::None)
            .ch_layout(AudioChannelLayout::new(2).expect("channel_count is a valid value"))
            .build();

        // Safety: We are zeroing the memory for the encoder context.
        let mut encoder = unsafe { std::mem::zeroed::<AVCodecContext>() };
        let result = settings.apply(&mut encoder);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            FfmpegError::Arguments("sample_rate, channel_layout and sample_fmt must be set")
        );
    }

    #[test]
    fn test_encoder_settings_apply_video() {
        let sample_aspect_ratio = AVRational { num: 1, den: 1 };
        let video_settings = VideoEncoderSettings::builder()
            .width(1920)
            .height(1080)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .sample_aspect_ratio(sample_aspect_ratio.into())
            .gop_size(12)
            .build();

        // Safety: We are zeroing the memory for the encoder context.
        let mut encoder = unsafe { std::mem::zeroed::<AVCodecContext>() };
        let encoder_settings = EncoderSettings::Video(video_settings);
        let result = encoder_settings.apply(&mut encoder);

        assert!(result.is_ok(), "Failed to apply video settings: {:?}", result.err());
        assert_eq!(encoder.width, 1920);
        assert_eq!(encoder.height, 1080);
        assert_eq!(AVPixelFormat(encoder.pix_fmt), AVPixelFormat::Yuv420p);
        assert_eq!(Rational::from(encoder.sample_aspect_ratio), sample_aspect_ratio.into());
    }

    #[test]
    fn test_encoder_settings_apply_audio() {
        let audio_settings = AudioEncoderSettings::builder()
            .sample_rate(44100)
            .sample_fmt(AVSampleFormat::Fltp)
            .ch_layout(AudioChannelLayout::new(2).expect("channel_count is a valid value"))
            .thread_count(4)
            .build();

        // Safety: We are zeroing the memory for the encoder context.
        let mut encoder = unsafe { std::mem::zeroed::<AVCodecContext>() };
        let encoder_settings = EncoderSettings::Audio(audio_settings);
        let result = encoder_settings.apply(&mut encoder);

        assert!(result.is_ok(), "Failed to apply audio settings: {:?}", result.err());
        assert_eq!(encoder.sample_rate, 44100);
        assert_eq!(AVSampleFormat(encoder.sample_fmt), AVSampleFormat::Fltp);
        assert_eq!(encoder.thread_count, 4);
    }

    #[test]
    fn test_encoder_settings_codec_specific_options() {
        let mut video_codec_options = Dictionary::new();
        video_codec_options.set(c"preset", c"fast").expect("Failed to set preset");

        let video_settings = VideoEncoderSettings::builder()
            .width(8)
            .height(8)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .codec_specific_options(video_codec_options.clone())
            .build();
        let mut encoder_settings = EncoderSettings::Video(video_settings);
        let options = encoder_settings.codec_specific_options();

        assert!(options.is_some());
        assert_eq!(options.unwrap().get(c"preset"), Some(c"fast"));

        let mut audio_codec_options = Dictionary::new();
        audio_codec_options.set(c"bitrate", c"128k").expect("Failed to set bitrate");
        let audio_settings = AudioEncoderSettings::builder()
            .sample_rate(44100)
            .sample_fmt(AVSampleFormat::Fltp)
            .ch_layout(AudioChannelLayout::new(2).expect("channel_count is a valid value"))
            .thread_count(4)
            .codec_specific_options(audio_codec_options)
            .build();
        let mut encoder_settings = EncoderSettings::Audio(audio_settings);
        let options = encoder_settings.codec_specific_options();

        assert!(options.is_some());
        assert_eq!(options.unwrap().get(c"bitrate"), Some(c"128k"));
    }

    #[test]
    fn test_from_video_encoder_settings() {
        let sample_aspect_ratio = AVRational { num: 1, den: 1 };
        let video_settings = VideoEncoderSettings::builder()
            .width(1920)
            .height(1080)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .sample_aspect_ratio(sample_aspect_ratio.into())
            .gop_size(12)
            .build();
        let encoder_settings: EncoderSettings = video_settings.into();

        if let EncoderSettings::Video(actual_video_settings) = encoder_settings {
            assert_eq!(actual_video_settings.width, 1920);
            assert_eq!(actual_video_settings.height, 1080);
            assert_eq!(actual_video_settings.frame_rate, 30.into());
            assert_eq!(actual_video_settings.pixel_format, AVPixelFormat::Yuv420p);
            assert_eq!(actual_video_settings.sample_aspect_ratio, Some(sample_aspect_ratio.into()));
            assert_eq!(actual_video_settings.gop_size, Some(12));
        } else {
            panic!("Expected EncoderSettings::Video variant");
        }
    }

    #[test]
    fn test_from_audio_encoder_settings() {
        let audio_settings = AudioEncoderSettings::builder()
            .sample_rate(44100)
            .sample_fmt(AVSampleFormat::Fltp)
            .ch_layout(AudioChannelLayout::new(2).expect("channel_count is a valid value"))
            .thread_count(4)
            .build();
        let encoder_settings: EncoderSettings = audio_settings.into();

        if let EncoderSettings::Audio(actual_audio_settings) = encoder_settings {
            assert_eq!(actual_audio_settings.sample_rate, 44100);
            assert_eq!(actual_audio_settings.sample_fmt, AVSampleFormat::Fltp);
            assert_eq!(actual_audio_settings.thread_count, Some(4));
        } else {
            panic!("Expected EncoderSettings::Audio variant");
        }
    }

    #[test]
    fn test_encoder_new_with_null_codec() {
        let codec = EncoderCodec::empty();
        let data = std::io::Cursor::new(Vec::new());
        let options = OutputOptions::builder().format_name("mp4").unwrap().build();
        let mut output = Output::new(data, options).expect("Failed to create Output");
        let incoming_time_base = AVRational { num: 1, den: 1000 };
        let outgoing_time_base = AVRational { num: 1, den: 1000 };
        let settings = VideoEncoderSettings::builder()
            .width(0)
            .height(0)
            .pixel_format(AVPixelFormat::Yuv420p)
            .frame_rate(0.into())
            .build();
        let result = Encoder::new(codec, &mut output, incoming_time_base, outgoing_time_base, settings);

        assert!(matches!(result, Err(FfmpegError::NoEncoder)));
    }

    #[test]
    fn test_encoder_new_success() {
        let codec = EncoderCodec::new(AVCodecID::Mpeg4);
        assert!(codec.is_some(), "Failed to find MPEG-4 encoder");
        let data = std::io::Cursor::new(Vec::new());
        let options = OutputOptions::builder().format_name("mp4").unwrap().build();
        let mut output = Output::new(data, options).expect("Failed to create Output");
        let incoming_time_base = AVRational { num: 1, den: 1000 };
        let outgoing_time_base = AVRational { num: 1, den: 1000 };
        let settings = VideoEncoderSettings::builder()
            .width(1920)
            .height(1080)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .build();
        let result = Encoder::new(codec.unwrap(), &mut output, incoming_time_base, outgoing_time_base, settings);

        assert!(result.is_ok(), "Encoder creation failed: {:?}", result.err());

        let encoder = result.unwrap();
        assert_eq!(encoder.incoming_time_base, Rational::static_new::<1, 1000>());
        assert_eq!(encoder.outgoing_time_base, Rational::static_new::<1, 1000>());
        assert_eq!(encoder.stream_index, 0);
    }

    #[test]
    fn test_send_eof() {
        let codec = EncoderCodec::new(AVCodecID::Mpeg4).expect("Failed to find MPEG-4 encoder");
        let data = std::io::Cursor::new(Vec::new());
        let options = OutputOptions::builder().format_name("mp4").unwrap().build();
        let mut output = Output::new(data, options).expect("Failed to create Output");
        let video_settings = VideoEncoderSettings::builder()
            .width(640)
            .height(480)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .build();
        let mut encoder = Encoder::new(
            codec,
            &mut output,
            AVRational { num: 1, den: 1000 },
            AVRational { num: 1, den: 1000 },
            video_settings,
        )
        .expect("Failed to create encoder");

        let result = encoder.send_eof();
        assert!(result.is_ok(), "send_eof returned an error: {:?}", result.err());
        assert!(encoder.send_eof().is_err(), "send_eof should return an error");
    }

    #[test]
    fn test_encoder_getters() {
        let codec = EncoderCodec::new(AVCodecID::Mpeg4).expect("Failed to find MPEG-4 encoder");
        let data = std::io::Cursor::new(Vec::new());
        let options = OutputOptions::builder().format_name("mp4").unwrap().build();
        let mut output = Output::new(data, options).expect("Failed to create Output");
        let incoming_time_base = AVRational { num: 1, den: 1000 };
        let outgoing_time_base = AVRational { num: 1, den: 1000 };
        let video_settings = VideoEncoderSettings::builder()
            .width(640)
            .height(480)
            .frame_rate(30.into())
            .pixel_format(AVPixelFormat::Yuv420p)
            .build();
        let encoder = Encoder::new(codec, &mut output, incoming_time_base, outgoing_time_base, video_settings)
            .expect("Failed to create encoder");

        let stream_index = encoder.stream_index();
        assert_eq!(stream_index, 0, "Unexpected stream index: expected 0, got {stream_index}");

        let actual_incoming_time_base = encoder.incoming_time_base();
        assert_eq!(
            actual_incoming_time_base,
            incoming_time_base.into(),
            "Unexpected incoming_time_base: expected {incoming_time_base:?}, got {actual_incoming_time_base:?}"
        );

        let actual_outgoing_time_base = encoder.outgoing_time_base();
        assert_eq!(
            actual_outgoing_time_base,
            outgoing_time_base.into(),
            "Unexpected outgoing_time_base: expected {outgoing_time_base:?}, got {actual_outgoing_time_base:?}"
        );
    }

    #[test]
    fn test_encoder_encode_video() {
        let mut input = Input::open("../../assets/avc_aac.mp4").expect("Failed to open input file");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("No video stream found");
        let mut decoder = Decoder::new(&video_stream)
            .expect("Failed to create decoder")
            .video()
            .expect("Failed to create video decoder");
        let mut output = Output::seekable(
            std::io::Cursor::new(Vec::new()),
            OutputOptions::builder().format_name("mpegts").unwrap().build(),
        )
        .expect("Failed to create Output");
        let mut encoder = Encoder::new(
            EncoderCodec::new(AVCodecID::Mpeg4).expect("Failed to find MPEG-4 encoder"),
            &mut output,
            AVRational { num: 1, den: 1000 },
            video_stream.time_base(),
            VideoEncoderSettings::builder()
                .width(decoder.width())
                .height(decoder.height())
                .frame_rate(decoder.frame_rate())
                .pixel_format(decoder.pixel_format())
                .build(),
        )
        .expect("Failed to create encoder");

        output.write_header().expect("Failed to write header");

        let input_stream_index = video_stream.index();

        while let Some(packet) = input.receive_packet().expect("Failed to receive packet") {
            if packet.stream_index() == input_stream_index {
                decoder.send_packet(&packet).expect("Failed to send packet");
                while let Some(frame) = decoder.receive_frame().expect("Failed to receive frame") {
                    encoder.send_frame(&frame).expect("Failed to send frame");
                    while let Some(packet) = encoder.receive_packet().expect("Failed to receive packet") {
                        output.write_packet(&packet).expect("Failed to write packet");
                    }
                }
            }
        }

        encoder.send_eof().expect("Failed to send EOF");
        while let Some(packet) = encoder.receive_packet().expect("Failed to receive packet") {
            output.write_packet(&packet).expect("Failed to write packet");
        }

        output.write_trailer().expect("Failed to write trailer");

        let mut cursor = std::io::Cursor::new(Bytes::from(output.into_inner().into_inner()));
        let mut boxes = Vec::new();
        while cursor.has_remaining() {
            let mut _box = scuffle_mp4::DynBox::demux(&mut cursor).expect("Failed to demux box");
            match &mut _box {
                scuffle_mp4::DynBox::Mdat(mdat) => {
                    mdat.data.iter_mut().for_each(|buf| {
                        let mut hash = sha2::Sha256::new();
                        hash.write_all(buf).unwrap();
                        *buf = Bytes::new();
                    });
                }
                scuffle_mp4::DynBox::Moov(moov) => {
                    moov.traks.iter_mut().for_each(|trak| {
                        // these can change from version to version
                        trak.mdia.minf.stbl.stsd.entries.clear();
                        if let Some(stsz) = trak.mdia.minf.stbl.stsz.as_mut() {
                            stsz.samples.clear();
                        }
                        trak.mdia.minf.stbl.stco.entries.clear();
                    });
                }
                _ => {}
            }
            boxes.push(_box);
        }
        insta::assert_debug_snapshot!("test_encoder_encode_video", &boxes);
    }

    /// make sure [#248](https://github.com/ScuffleCloud/scuffle/pull/248) doesn't happen again
    #[test]
    fn test_pr_248() {
        let mut output = Output::seekable(
            std::io::Cursor::new(Vec::new()),
            OutputOptions::builder().format_name("mpegts").unwrap().build(),
        )
        .expect("Failed to create Output");

        let mut settings = Dictionary::new();
        settings.set(c"key", c"value").expect("Failed to set Dictionary entry");

        let codec = EncoderCodec::new(AVCodecID::Mpeg4).expect("Missing MPEG-4 codec");

        Encoder::new(
            codec,
            &mut output,
            AVRational { num: 1, den: 100 },
            AVRational { num: 1, den: 100 },
            VideoEncoderSettings::builder()
                .width(16)
                .height(16)
                .frame_rate(30.into())
                .pixel_format(AVPixelFormat::Yuv420p)
                .codec_specific_options(settings)
                .build(),
        )
        .expect("Failed to create new Encoder");
    }

    #[test]
    fn test_encoder_low_latency_x264() {
        let codec = EncoderCodec::by_name("libx264").expect("libx264 encoder should be available");

        let mut input = Input::open("../../assets/avc_aac.mp4").expect("Failed to open input file");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("No video stream found");
        let input_stream_index = video_stream.index();

        let mut decoder = Decoder::new(&video_stream)
            .expect("Failed to create decoder")
            .video()
            .expect("Failed to create video decoder");

        let mut output = Output::seekable(
            std::io::Cursor::new(Vec::new()),
            OutputOptions::builder().format_name("mpegts").unwrap().build(),
        )
        .expect("Failed to create Output");

        let mut codec_options = Dictionary::new();
        codec_options.set("preset", "ultrafast").unwrap();
        codec_options.set("tune", "zerolatency").unwrap();

        let bitrate = 1_000_000i64;

        let mut encoder = Encoder::new(
            codec,
            &mut output,
            AVRational { num: 1, den: 1000 },
            video_stream.time_base(),
            VideoEncoderSettings::builder()
                .width(decoder.width())
                .height(decoder.height())
                .frame_rate(decoder.frame_rate())
                .pixel_format(decoder.pixel_format())
                .max_b_frames(0)
                .bitrate(bitrate)
                .rc_max_rate(bitrate)
                .rc_buffer_size(bitrate as i32)
                .codec_specific_options(codec_options)
                .low_delay(true)
                .build(),
        )
        .expect("Failed to create encoder");

        output.write_header().expect("Failed to write header");

        let mut frames_sent = 0usize;
        let mut packets_received = 0usize;

        while let Some(packet) = input.receive_packet().expect("Failed to receive packet") {
            if packet.stream_index() != input_stream_index {
                continue;
            }
            decoder.send_packet(&packet).expect("Failed to send packet");
            while let Some(frame) = decoder.receive_frame().expect("Failed to receive frame") {
                encoder.send_frame(&frame).expect("Failed to send frame");
                frames_sent += 1;

                // With zerolatency tune, each frame should produce a packet immediately.
                while let Some(pkt) = encoder.receive_packet().expect("Failed to receive packet") {
                    packets_received += 1;
                    output.write_packet(&pkt).expect("Failed to write packet");
                }
            }
        }

        // Drain any remaining packets.
        encoder.send_eof().expect("Failed to send EOF");
        while let Some(pkt) = encoder.receive_packet().expect("Failed to receive packet") {
            packets_received += 1;
            output.write_packet(&pkt).expect("Failed to write packet");
        }

        output.write_trailer().expect("Failed to write trailer");

        // With zerolatency tune + bframes=0, every frame should produce a
        // packet immediately — so total packets == total frames.
        assert_eq!(
            packets_received, frames_sent,
            "Expected one packet per frame with zerolatency tune, got {packets_received} packets for {frames_sent} frames",
        );
        assert!(frames_sent > 0, "Should have encoded at least one frame");
    }

    /// Per-frame packet output with zerolatency x264 and 144p scaling.
    /// Matches the transcoder's exact config: mpegts output, no write_header,
    /// no bitrate (CRF mode), tune=zerolatency, max_b_frames=0.
    #[test]
    fn test_encoder_per_frame_packet_output_144p() {
        use crate::scaler::VideoScaler;

        let codec = EncoderCodec::by_name("libx264").expect("libx264 encoder should be available");

        let mut input = Input::open("../../assets/avc_aac.mp4").expect("Failed to open input file");
        let streams = input.streams();
        let video_stream = streams.best(AVMediaType::Video).expect("No video stream found");
        let input_stream_index = video_stream.index();

        let mut decoder = Decoder::new(&video_stream)
            .expect("Failed to create decoder")
            .video()
            .expect("Failed to create video decoder");

        let out_w = 256;
        let out_h = 144;
        let mut scaler = VideoScaler::new(
            decoder.width(),
            decoder.height(),
            decoder.pixel_format(),
            out_w,
            out_h,
            AVPixelFormat::Yuv420p,
        )
        .expect("Failed to create scaler");

        let mut output = Output::seekable(
            std::io::Cursor::new(Vec::new()),
            OutputOptions::builder().format_name("mpegts").unwrap().build(),
        )
        .expect("Failed to create Output");

        let mut codec_options = Dictionary::new();
        codec_options.set("preset", "ultrafast").unwrap();
        codec_options.set("tune", "zerolatency").unwrap();

        let mut encoder = Encoder::new(
            codec,
            &mut output,
            video_stream.time_base(),
            AVRational { num: 1, den: 90_000 }, // MPEGTS timebase
            VideoEncoderSettings::builder()
                .width(out_w)
                .height(out_h)
                .frame_rate(decoder.frame_rate())
                .pixel_format(AVPixelFormat::Yuv420p)
                .gop_size(decoder.frame_rate().as_f64().ceil() as i32)
                .max_b_frames(0)
                .codec_specific_options(codec_options)
                .low_delay(true)
                .build(),
        )
        .expect("Failed to create encoder");

        // No write_header — matching the transcoder's dummy output pattern.

        let mut packets_per_frame: Vec<usize> = Vec::new();
        let max_frames = 10;

        'outer: while let Some(packet) = input.receive_packet().expect("receive_packet") {
            if packet.stream_index() != input_stream_index {
                continue;
            }
            decoder.send_packet(&packet).expect("send_packet");
            while let Some(frame) = decoder.receive_frame().expect("receive_frame") {
                let scaled = scaler.process(&frame).expect("scale");
                encoder.send_frame(scaled).expect("send_frame");

                let mut count = 0usize;
                while let Some(_pkt) = encoder.receive_packet().expect("receive_packet") {
                    count += 1;
                }
                packets_per_frame.push(count);

                if packets_per_frame.len() >= max_frames {
                    break 'outer;
                }
            }
        }

        // With tune=zerolatency + bframes=0, every frame must produce
        // exactly 1 packet — including the very first one.
        for (i, &count) in packets_per_frame.iter().enumerate() {
            assert_eq!(
                count, 1,
                "Frame {i} produced {count} packets, expected 1 (per-frame: {packets_per_frame:?})",
            );
        }
    }
}
