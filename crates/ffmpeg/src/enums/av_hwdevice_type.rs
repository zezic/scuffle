use nutype_enum::nutype_enum;

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVHWDeviceType>() == std::mem::size_of_val(&AV_HWDEVICE_TYPE_NONE));
};

nutype_enum! {
    /// Represents the different device types used for hardware decoding
    ///
    /// See FFmpeg's `AVHWDeviceType` in the official documentation:
    /// <https://ffmpeg.org/doxygen/trunk/hwcontext_8h.html#acf25724be4b066a51ad86aa9214b0d34>
    pub enum AVHWDeviceType(i32) {
        /// No device type
        None = AV_HWDEVICE_TYPE_NONE as _,
        /// VDPAU device type
        VDPAU = AV_HWDEVICE_TYPE_VDPAU as _,
        /// CUDA device type
        CUDA = AV_HWDEVICE_TYPE_CUDA as _,
        /// VAAPI device type
        VAAPI = AV_HWDEVICE_TYPE_VAAPI as _,
        /// DXVA2 device type
        DXVA2 = AV_HWDEVICE_TYPE_DXVA2 as _,
        /// QSV device type
        QSV = AV_HWDEVICE_TYPE_QSV as _,
        /// VideoToolbox device type
        VIDEOTOOLBOX = AV_HWDEVICE_TYPE_VIDEOTOOLBOX as _,
        /// D3D11VA device type
        D3D11VA = AV_HWDEVICE_TYPE_D3D11VA as _,
        /// DRM device type
        DRM = AV_HWDEVICE_TYPE_DRM as _,
        /// OpenCL device type
        OPENCL = AV_HWDEVICE_TYPE_OPENCL as _,
        /// MediaCodec device type
        MEDIACODEC = AV_HWDEVICE_TYPE_MEDIACODEC as _,
        /// Vulkan device type
        VULKAN = AV_HWDEVICE_TYPE_VULKAN as _,
        /// D3D12VA device type
        D3D12VA = AV_HWDEVICE_TYPE_D3D12VA as _,
    }
}

impl PartialEq<i32> for AVHWDeviceType {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVHWDeviceType {
    fn from(value: u32) -> Self {
        AVHWDeviceType(value as i32)
    }
}

impl From<AVHWDeviceType> for u32 {
    fn from(value: AVHWDeviceType) -> Self {
        value.0 as u32
    }
}
