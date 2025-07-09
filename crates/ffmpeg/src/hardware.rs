//! Hardware acceleration context management for FFmpeg.
//!
//! This module provides functionality for creating and managing hardware
//! acceleration contexts, particularly for CUDA-based decoding.

use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::frame::{GenericFrame, VideoFrame};
use crate::smart_object::SmartPtr;
use crate::{AVHWDeviceType, AVPixelFormat};

/// A hardware device context for hardware acceleration.
pub struct HardwareContext {
    device_ctx: SmartPtr<AVBufferRef>,
    device_type: AVHWDeviceType,
    hw_pixel_format: AVPixelFormat,
}

impl HardwareContext {
    /// Create a new hardware context for the specified device type.
    ///
    /// # Arguments
    /// * `device_type` - The hardware device type (e.g., CUDA, VAAPI)
    /// * `device` - Optional device specifier (e.g., "/dev/dri/renderD128" for VAAPI)
    ///
    /// # Returns
    /// A new `HardwareContext` on success, or an error if the hardware device
    /// could not be created.
    pub fn new(device_type: AVHWDeviceType, device: Option<&str>) -> Result<Self, FfmpegError> {
        let device_cstr = device
            .map(|d| std::ffi::CString::new(d))
            .transpose()
            .map_err(|_| FfmpegError::Code(FfmpegErrorCode::InvalidData))?;

        let device_ptr = device_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());

        let mut hw_device_ctx = std::ptr::null_mut();

        // Safety: av_hwdevice_ctx_create is safe to call with valid parameters
        let ret =
            unsafe { av_hwdevice_ctx_create(&mut hw_device_ctx, device_type.into(), device_ptr, std::ptr::null_mut(), 0) };

        FfmpegErrorCode(ret).result()?;

        let destructor = |ptr: &mut *mut AVBufferRef| {
            // Safety: av_buffer_unref is safe to call with a valid pointer
            unsafe { av_buffer_unref(ptr) };
        };

        // Safety: hw_device_ctx is a valid pointer from av_hwdevice_ctx_create
        let device_ctx = unsafe { SmartPtr::wrap_non_null(hw_device_ctx, destructor) }.ok_or(FfmpegError::Alloc)?;

        // Determine the hardware pixel format based on device type
        let hw_pixel_format = match device_type {
            AVHWDeviceType::Cuda => AVPixelFormat::Cuda,
            AVHWDeviceType::Vaapi => AVPixelFormat::Vaapi,
            _ => return Err(FfmpegError::NoDecoder),
        };

        Ok(Self {
            device_ctx,
            device_type,
            hw_pixel_format,
        })
    }

    /// Get the hardware device type.
    pub fn device_type(&self) -> AVHWDeviceType {
        self.device_type
    }

    /// Get the hardware pixel format.
    pub fn hw_pixel_format(&self) -> AVPixelFormat {
        self.hw_pixel_format
    }

    /// Get a reference to the underlying AVBufferRef.
    pub(crate) fn as_ptr(&self) -> *mut AVBufferRef {
        self.device_ctx.as_ptr() as *mut AVBufferRef
    }

    /// Create a reference to this hardware context.
    pub(crate) fn reference(&self) -> Result<SmartPtr<AVBufferRef>, FfmpegError> {
        // Safety: av_buffer_ref is safe to call with a valid pointer
        let ref_ptr = unsafe { av_buffer_ref(self.device_ctx.as_ptr() as *mut AVBufferRef) };

        let destructor = |ptr: &mut *mut AVBufferRef| {
            // Safety: av_buffer_unref is safe to call with a valid pointer
            unsafe { av_buffer_unref(ptr) };
        };

        // Safety: ref_ptr is a valid pointer from av_buffer_ref
        unsafe { SmartPtr::wrap_non_null(ref_ptr, destructor) }.ok_or(FfmpegError::Alloc)
    }

    /// Transfer data from a hardware frame to system memory.
    ///
    /// This function is used to copy frame data from GPU memory to CPU memory
    /// for hardware-accelerated frames.
    ///
    /// # Arguments
    /// * `hw_frame` - The hardware frame to transfer data from
    ///
    /// # Returns
    /// A new `VideoFrame` containing the transferred data in system memory.
    pub fn transfer_data_from_hw(&self, hw_frame: &VideoFrame) -> Result<VideoFrame, FfmpegError> {
        // Safety: av_frame_alloc is safe to call
        let sw_frame_ptr = unsafe { av_frame_alloc() };
        if sw_frame_ptr.is_null() {
            return Err(FfmpegError::Alloc);
        }

        let destructor = |ptr: &mut *mut AVFrame| {
            // Safety: av_frame_free is safe to call with a valid pointer
            unsafe { av_frame_free(ptr) };
        };

        // Safety: sw_frame_ptr is a valid pointer from av_frame_alloc
        let mut sw_frame = unsafe { SmartPtr::wrap_non_null(sw_frame_ptr, destructor) }.ok_or(FfmpegError::Alloc)?;

        // Safety: av_hwframe_transfer_data is safe to call with valid frame pointers
        let ret = unsafe { av_hwframe_transfer_data(sw_frame.as_mut_ptr(), hw_frame.as_ptr(), 0) };

        FfmpegErrorCode(ret).result()?;

        // Safety: sw_frame is a valid AVFrame pointer
        Ok(unsafe { GenericFrame::wrap(sw_frame.as_ptr() as *mut AVFrame) }
            .ok_or(FfmpegError::Alloc)?
            .video())
    }

    /// Check if a frame is a hardware frame.
    ///
    /// # Arguments
    /// * `frame` - The frame to check
    ///
    /// # Returns
    /// `true` if the frame is a hardware frame, `false` otherwise.
    pub fn is_hw_frame(&self, frame: &VideoFrame) -> bool {
        frame.format() == self.hw_pixel_format
    }
}

/// Hardware decoder configuration.
#[derive(Debug, Clone)]
pub struct HardwareConfig {
    /// The hardware device type to use.
    pub device_type: AVHWDeviceType,
    /// Optional device specifier.
    pub device: Option<String>,
    /// Whether to automatically transfer frames to system memory.
    pub auto_transfer: bool,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            device_type: AVHWDeviceType::None,
            device: None,
            auto_transfer: false,
        }
    }
}

impl HardwareConfig {
    /// Create a new hardware configuration for CUDA.
    pub fn cuda() -> Self {
        Self {
            device_type: AVHWDeviceType::Cuda,
            device: None,
            auto_transfer: true,
        }
    }

    /// Create a new hardware configuration for VA-API.
    pub fn vaapi() -> Self {
        Self {
            device_type: AVHWDeviceType::Vaapi,
            device: None,
            auto_transfer: true,
        }
    }

    /// Set the device specifier.
    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }

    /// Set whether to automatically transfer frames to system memory.
    pub fn with_auto_transfer(mut self, auto_transfer: bool) -> Self {
        self.auto_transfer = auto_transfer;
        self
    }

    /// Check if hardware acceleration is enabled.
    pub fn is_hardware_accelerated(&self) -> bool {
        self.device_type != AVHWDeviceType::None
    }
}

/// Callback function for hardware pixel format selection.
///
/// This function is called by FFmpeg to select the appropriate hardware pixel format
/// from a list of available formats.
pub(crate) unsafe extern "C" fn get_hw_format(ctx: *mut AVCodecContext, pix_fmts: *const i32) -> i32 {
    // Get the hardware pixel format from the context's opaque data
    if ctx.is_null() {
        return AV_PIX_FMT_NONE;
    }

    let opaque = unsafe { (*ctx).opaque };
    if opaque.is_null() {
        return AV_PIX_FMT_NONE;
    }

    let hw_pix_fmt = unsafe { *(opaque as *const i32) };

    // Search for the hardware format in the list of available formats
    let mut p = pix_fmts;
    while !p.is_null() && unsafe { *p } != AV_PIX_FMT_NONE {
        if unsafe { *p } == hw_pix_fmt {
            return unsafe { *p };
        }
        p = unsafe { p.add(1) };
    }

    // If hardware format not found, return none
    AV_PIX_FMT_NONE
}

/// Find the hardware configuration for a codec that matches the specified device type.
///
/// # Arguments
/// * `codec` - The codec to check for hardware support
/// * `device_type` - The desired hardware device type
///
/// # Returns
/// The hardware pixel format if supported, otherwise `None`.
pub(crate) fn find_hw_config_for_codec(codec: *const AVCodec, device_type: AVHWDeviceType) -> Option<AVPixelFormat> {
    if codec.is_null() {
        return None;
    }

    let mut i = 0;
    loop {
        // Safety: avcodec_get_hw_config is safe to call with a valid codec pointer
        let config = unsafe { avcodec_get_hw_config(codec, i) };
        if config.is_null() {
            break;
        }

        // Safety: config is a valid pointer from avcodec_get_hw_config
        let config_ref = unsafe { &*config };

        if (config_ref.methods & AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX as i32) != 0
            && config_ref.device_type == u32::from(device_type)
        {
            return Some(AVPixelFormat::from(config_ref.pix_fmt as u32));
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_config_default() {
        let config = HardwareConfig::default();
        assert_eq!(config.device_type, AVHWDeviceType::None);
        assert!(!config.is_hardware_accelerated());
    }

    #[test]
    fn test_hardware_config_cuda() {
        let config = HardwareConfig::cuda();
        assert_eq!(config.device_type, AVHWDeviceType::Cuda);
        assert!(config.is_hardware_accelerated());
        assert!(config.auto_transfer);
    }

    #[test]
    fn test_hardware_config_vaapi() {
        let config = HardwareConfig::vaapi();
        assert_eq!(config.device_type, AVHWDeviceType::Vaapi);
        assert!(config.is_hardware_accelerated());
        assert!(config.auto_transfer);
    }

    #[test]
    fn test_hardware_config_with_device() {
        let config = HardwareConfig::cuda().with_device("/dev/nvidia0");
        assert_eq!(config.device, Some("/dev/nvidia0".to_string()));
    }

    #[test]
    fn test_hardware_config_with_auto_transfer() {
        let config = HardwareConfig::cuda().with_auto_transfer(false);
        assert!(!config.auto_transfer);
    }

    #[test]
    fn test_hw_device_type_iter() {
        let types: Vec<_> = AVHWDeviceType::iter_types().collect();
        assert!(!types.is_empty());
        assert!(!types.contains(&AVHWDeviceType::None));
    }

    #[test]
    fn test_hw_device_type_find_by_name() {
        assert_eq!(AVHWDeviceType::find_by_name("cuda"), Some(AVHWDeviceType::Cuda));
        assert_eq!(AVHWDeviceType::find_by_name("vaapi"), Some(AVHWDeviceType::Vaapi));
        assert_eq!(AVHWDeviceType::find_by_name("nonexistent"), None);
    }

    #[test]
    fn test_hw_device_type_name() {
        assert_eq!(AVHWDeviceType::Cuda.name(), Some("cuda"));
        assert_eq!(AVHWDeviceType::Vaapi.name(), Some("vaapi"));
        // None type may not have a name in FFmpeg
        let none_name = AVHWDeviceType::None.name();
        assert!(none_name.is_none() || none_name == Some("none"));
    }

    #[test]
    fn test_hw_device_type_display() {
        assert_eq!(format!("{}", AVHWDeviceType::Cuda), "cuda");
        assert_eq!(format!("{}", AVHWDeviceType::Vaapi), "vaapi");
        assert_eq!(format!("{}", AVHWDeviceType::None), "none");
    }
}
