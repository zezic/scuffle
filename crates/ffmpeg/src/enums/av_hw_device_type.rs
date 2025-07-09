use nutype_enum::nutype_enum;

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVHWDeviceType>() == std::mem::size_of_val(&AV_HWDEVICE_TYPE_NONE));
};

nutype_enum! {
    /// Hardware device types used in FFmpeg's hardware acceleration.
    ///
    /// This enum represents different hardware acceleration backends
    /// that can be used for decoding and encoding operations.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/hwcontext_8h.html>
    pub enum AVHWDeviceType(u32) {
        /// No hardware acceleration.
        /// Corresponds to `AV_HWDEVICE_TYPE_NONE`.
        None = AV_HWDEVICE_TYPE_NONE as _,

        /// VDPAU hardware acceleration.
        /// Video Decode and Presentation API for Unix (NVIDIA).
        /// Corresponds to `AV_HWDEVICE_TYPE_VDPAU`.
        Vdpau = AV_HWDEVICE_TYPE_VDPAU as _,

        /// CUDA hardware acceleration.
        /// NVIDIA's Compute Unified Device Architecture.
        /// Corresponds to `AV_HWDEVICE_TYPE_CUDA`.
        Cuda = AV_HWDEVICE_TYPE_CUDA as _,

        /// VA-API hardware acceleration.
        /// Video Acceleration API (Intel, AMD).
        /// Corresponds to `AV_HWDEVICE_TYPE_VAAPI`.
        Vaapi = AV_HWDEVICE_TYPE_VAAPI as _,

        /// DXVA2 hardware acceleration.
        /// DirectX Video Acceleration 2 (Windows).
        /// Corresponds to `AV_HWDEVICE_TYPE_DXVA2`.
        Dxva2 = AV_HWDEVICE_TYPE_DXVA2 as _,

        /// Intel Quick Sync Video.
        /// Hardware acceleration for Intel processors.
        /// Corresponds to `AV_HWDEVICE_TYPE_QSV`.
        Qsv = AV_HWDEVICE_TYPE_QSV as _,

        /// VideoToolbox hardware acceleration.
        /// Apple's hardware acceleration framework (macOS/iOS).
        /// Corresponds to `AV_HWDEVICE_TYPE_VIDEOTOOLBOX`.
        Videotoolbox = AV_HWDEVICE_TYPE_VIDEOTOOLBOX as _,

        /// D3D11VA hardware acceleration.
        /// Direct3D 11 Video Acceleration (Windows).
        /// Corresponds to `AV_HWDEVICE_TYPE_D3D11VA`.
        D3d11va = AV_HWDEVICE_TYPE_D3D11VA as _,

        /// DRM hardware acceleration.
        /// Direct Rendering Manager (Linux).
        /// Corresponds to `AV_HWDEVICE_TYPE_DRM`.
        Drm = AV_HWDEVICE_TYPE_DRM as _,

        /// OpenCL hardware acceleration.
        /// Open Computing Language.
        /// Corresponds to `AV_HWDEVICE_TYPE_OPENCL`.
        Opencl = AV_HWDEVICE_TYPE_OPENCL as _,

        /// MediaCodec hardware acceleration.
        /// Android's multimedia codec framework.
        /// Corresponds to `AV_HWDEVICE_TYPE_MEDIACODEC`.
        Mediacodec = AV_HWDEVICE_TYPE_MEDIACODEC as _,

        /// Vulkan hardware acceleration.
        /// Vulkan compute API.
        /// Corresponds to `AV_HWDEVICE_TYPE_VULKAN`.
        Vulkan = AV_HWDEVICE_TYPE_VULKAN as _,

        /// D3D12VA hardware acceleration.
        /// Direct3D 12 Video Acceleration (Windows).
        /// Corresponds to `AV_HWDEVICE_TYPE_D3D12VA`.
        D3d12va = AV_HWDEVICE_TYPE_D3D12VA as _,
    }
}

impl Default for AVHWDeviceType {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for AVHWDeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match *self {
            Self::None => "none",
            Self::Vdpau => "vdpau",
            Self::Cuda => "cuda",
            Self::Vaapi => "vaapi",
            Self::Dxva2 => "dxva2",
            Self::Qsv => "qsv",
            Self::Videotoolbox => "videotoolbox",
            Self::D3d11va => "d3d11va",
            Self::Drm => "drm",
            Self::Opencl => "opencl",
            Self::Mediacodec => "mediacodec",
            Self::Vulkan => "vulkan",
            Self::D3d12va => "d3d12va",
            _ => "unknown",
        };
        write!(f, "{}", name)
    }
}

impl AVHWDeviceType {
    /// Find a hardware device type by name.
    ///
    /// Returns `None` if the name is not recognized.
    pub fn find_by_name(name: &str) -> Option<Self> {
        let c_name = std::ffi::CString::new(name).ok()?;

        // Safety: av_hwdevice_find_type_by_name is safe to call with a valid C string
        let device_type = unsafe { av_hwdevice_find_type_by_name(c_name.as_ptr()) };

        if device_type == AV_HWDEVICE_TYPE_NONE {
            None
        } else {
            Some(Self(device_type))
        }
    }

    /// Get the name of the hardware device type.
    ///
    /// Returns `None` if the device type is not recognized.
    pub fn name(&self) -> Option<&'static str> {
        // Safety: av_hwdevice_get_type_name is safe to call with a valid device type
        let name_ptr = unsafe { av_hwdevice_get_type_name(self.0) };

        if name_ptr.is_null() {
            None
        } else {
            // Safety: The returned pointer is valid for the lifetime of the program
            let c_str = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
            c_str.to_str().ok()
        }
    }

    /// Iterate over all available hardware device types.
    ///
    /// Returns an iterator that yields all supported hardware device types.
    pub fn iter_types() -> impl Iterator<Item = Self> {
        std::iter::successors(Some(Self::None), |&prev| {
            // Safety: av_hwdevice_iterate_types is safe to call
            let next = unsafe { av_hwdevice_iterate_types(prev.0) };
            if next == AV_HWDEVICE_TYPE_NONE {
                None
            } else {
                Some(Self(next))
            }
        })
        .skip(1) // Skip the initial None value
    }
}
