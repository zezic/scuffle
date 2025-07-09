use nutype_enum::nutype_enum;

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVPixelFormat>() == std::mem::size_of_val(&AV_PIX_FMT_NONE));
};

nutype_enum! {
    /// Pixel formats used in FFmpeg's `AVPixelFormat` enumeration.
    ///
    /// This enum represents different ways pixels can be stored in memory,
    /// including packed, planar, and hardware-accelerated formats.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/pixfmt_8h.html>
    pub enum AVPixelFormat(i32) {
        /// No pixel format specified or unknown format.
        /// Corresponds to `AV_PIX_FMT_NONE`.
        None = AV_PIX_FMT_NONE as _,

        /// Planar YUV 4:2:0 format, 12 bits per pixel.
        /// Each plane is stored separately, with 1 Cr & Cb sample per 2x2 Y samples.
        /// Corresponds to `AV_PIX_FMT_YUV420P`.
        Yuv420p = AV_PIX_FMT_YUV420P as _,

        /// Packed YUV 4:2:2 format, 16 bits per pixel.
        /// Stored as Y0 Cb Y1 Cr.
        /// Corresponds to `AV_PIX_FMT_Yuyv422`.
        Yuyv422 = AV_PIX_FMT_YUYV422 as _,

        /// Packed RGB format, 8 bits per channel (24bpp).
        /// Stored as RGBRGB...
        /// Corresponds to `AV_PIX_FMT_RGB24`.
        Rgb24 = AV_PIX_FMT_RGB24 as _,

        /// Packed BGR format, 8 bits per channel (24bpp).
        /// Stored as BGRBGR...
        /// Corresponds to `AV_PIX_FMT_BGR24`.
        Bgr24 = AV_PIX_FMT_BGR24 as _,

        /// Planar YUV 4:2:2 format, 16 bits per pixel.
        /// Each plane is stored separately, with 1 Cr & Cb sample per 2x1 Y samples.
        /// Corresponds to `AV_PIX_FMT_YUV422P`.
        Yuv422p = AV_PIX_FMT_YUV422P as _,

        /// Planar YUV 4:4:4 format, 24 bits per pixel.
        /// Each plane is stored separately, with 1 Cr & Cb sample per 1x1 Y samples.
        /// Corresponds to `AV_PIX_FMT_YUV444P`.
        Yuv444p = AV_PIX_FMT_YUV444P as _,

        /// Planar YUV 4:2:0 format with interleaved UV plane.
        /// 12 bits per pixel, Y plane followed by interleaved U and V samples.
        /// Corresponds to `AV_PIX_FMT_NV12`.
        Nv12 = AV_PIX_FMT_NV12 as _,

        /// 8-bit grayscale format, 8 bits per pixel.
        /// Corresponds to `AV_PIX_FMT_GRAY8`.
        Gray8 = AV_PIX_FMT_GRAY8 as _,

        /// 1-bit monochrome format, 0 is white, 1 is black.
        /// Pixels are stored in bytes, ordered from the most significant bit.
        /// Corresponds to `AV_PIX_FMT_MonoWhite`.
        MonoWhite = AV_PIX_FMT_MONOWHITE as _,

        /// 1-bit monochrome format, 0 is black, 1 is white.
        /// Pixels are stored in bytes, ordered from the most significant bit.
        /// Corresponds to `AV_PIX_FMT_MonoBlack`.
        MonoBlack = AV_PIX_FMT_MONOBLACK as _,

        /// Packed RGB 5:6:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGB565BE`
        Rgb565Be = AV_PIX_FMT_RGB565BE as _,

        /// Packed RGB 5:6:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGB565LE`
        Rgb565Le = AV_PIX_FMT_RGB565LE as _,

        /// Packed RGB 5:5:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGB555BE`
        Rgb555Be = AV_PIX_FMT_RGB555BE as _,

        /// Packed RGB 5:5:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGB555LE`
        Rgb555Le = AV_PIX_FMT_RGB555LE as _,

        /// Packed BGR 5:6:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_BGR565BE`
        Bgr565Be = AV_PIX_FMT_BGR565BE as _,

        /// Packed BGR 5:6:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_BGR565LE`
        Bgr565Le = AV_PIX_FMT_BGR565LE as _,

        /// Packed BGR 5:5:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_BGR555BE`
        Bgr555Be = AV_PIX_FMT_BGR555BE as _,

        /// Packed BGR 5:5:5 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_BGR555LE`
        Bgr555Le = AV_PIX_FMT_BGR555LE as _,

        /// Planar YUV 4:2:0 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_YUV420P16BE`
        Yuv420p16Be = AV_PIX_FMT_YUV420P16BE as _,

        /// Planar YUV 4:2:0 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_YUV420P16LE`
        Yuv420p16Le = AV_PIX_FMT_YUV420P16LE as _,

        /// Planar YUV 4:2:2 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_YUV422P16BE`
        Yuv422p16Be = AV_PIX_FMT_YUV422P16BE as _,

        /// Planar YUV 4:2:2 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_YUV422P16LE`
        Yuv422p16Le = AV_PIX_FMT_YUV422P16LE as _,

        /// Planar YUV 4:4:4 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_YUV444P16BE`
        Yuv444p16Be = AV_PIX_FMT_YUV444P16BE as _,

        /// Planar YUV 4:4:4 format, 16 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_YUV444P16LE`
        Yuv444p16Le = AV_PIX_FMT_YUV444P16LE as _,

        /// Packed RGB 16:16:16 format, 48 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGB48BE`
        Rgb48Be = AV_PIX_FMT_RGB48BE as _,

        /// Packed RGB 16:16:16 format, 48 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGB48LE`
        Rgb48Le = AV_PIX_FMT_RGB48LE as _,

        /// Packed RGBA 16:16:16:16 format, 64 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGBA64BE`
        Rgba64Be = AV_PIX_FMT_RGBA64BE as _,

        /// Packed RGBA 16:16:16:16 format, 64 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_RGBA64LE`
        Rgba64Le = AV_PIX_FMT_RGBA64LE as _,

        /// Packed BGRA 16:16:16:16 format, 64 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_BGRA64BE`
        Bgra64Be = AV_PIX_FMT_BGRA64BE as _,

        /// Packed BGRA 16:16:16:16 format, 64 bits per pixel.
        /// Corresponds to: `AV_PIX_FMT_BGRA64LE`
        Bgra64Le = AV_PIX_FMT_BGRA64LE as _,

        /// Hardware-accelerated format through VA-API.
        /// Corresponds to `AV_PIX_FMT_VAAPI`.
        Vaapi = AV_PIX_FMT_VAAPI as _,

        /// Hardware-accelerated format through CUDA.
        /// Corresponds to `AV_PIX_FMT_CUDA`.
        Cuda = AV_PIX_FMT_CUDA as _,

        /// Planar GBR format, 4:4:4 subsampling.
        /// Corresponds to `AV_PIX_FMT_GBRP`.
        Gbrp = AV_PIX_FMT_GBRP as _,

        /// Format count, not an actual pixel format.
        /// Used internally by FFmpeg.
        /// Corresponds to `AV_PIX_FMT_NB`.
        Nb = AV_PIX_FMT_NB as _,
    }
}

impl PartialEq<i32> for AVPixelFormat {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVPixelFormat {
    fn from(value: u32) -> Self {
        AVPixelFormat(value as i32)
    }
}

impl From<AVPixelFormat> for u32 {
    fn from(value: AVPixelFormat) -> Self {
        value.0 as u32
    }
}
