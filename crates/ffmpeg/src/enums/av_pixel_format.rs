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

        /// 8 bits with AV_PIX_FMT_RGB32 palette
        /// Corresponds to `AV_PIX_FMT_PAL8`.
        Pal8 = AV_PIX_FMT_PAL8 as _,

        /// Planar YUV 4:2:0, 12bpp, full scale (JPEG), deprecated
        /// Corresponds to `AV_PIX_FMT_YUVJ420P`.
        Yuvj420p = AV_PIX_FMT_YUVJ420P as _,

        /// Planar YUV 4:2:2, 16bpp, full scale (JPEG), deprecated
        /// Corresponds to `AV_PIX_FMT_YUVJ422P`.
        Yuvj422p = AV_PIX_FMT_YUVJ422P as _,

        /// Planar YUV 4:4:4, 24bpp, full scale (JPEG), deprecated
        /// Corresponds to `AV_PIX_FMT_YUVJ444P`.
        Yuvj444p = AV_PIX_FMT_YUVJ444P as _,

        /// Packed YUV 4:2:2, 16bpp, Cb Y0 Cr Y1
        /// Corresponds to `AV_PIX_FMT_UYVY422`.
        Uyvy422 = AV_PIX_FMT_UYVY422 as _,

        /// Planar YUV 4:1:0, 9bpp, (1 Cr & Cb sample per 4x4 Y samples)
        /// Corresponds to `AV_PIX_FMT_YUV410P`.
        Yuv410p = AV_PIX_FMT_YUV410P as _,

        /// Planar YUV 4:1:1, 12bpp, (1 Cr & Cb sample per 4x1 Y samples)
        /// Corresponds to `AV_PIX_FMT_YUV411P`.
        Yuv411p = AV_PIX_FMT_YUV411P as _,

        /// Packed YUV 4:1:1, 12bpp, Cb Y0 Y1 Cr Y2 Y3
        /// Corresponds to `AV_PIX_FMT_UYYVYY411`.
        Uyyvyy411 = AV_PIX_FMT_UYYVYY411 as _,

        /// Packed RGB 3:3:2, 8bpp, (msb)2B 3G 3R(lsb)
        /// Corresponds to `AV_PIX_FMT_BGR8`.
        Bgr8 = AV_PIX_FMT_BGR8 as _,

        /// Packed RGB 1:2:1 bitstream, 4bpp, (msb)1B 2G 1R(lsb)
        /// Corresponds to `AV_PIX_FMT_BGR4`.
        Bgr4 = AV_PIX_FMT_BGR4 as _,

        /// Packed RGB 1:2:1, 8bpp, (msb)1B 2G 1R(lsb)
        /// Corresponds to `AV_PIX_FMT_BGR4_BYTE`.
        Bgr4Byte = AV_PIX_FMT_BGR4_BYTE as _,

        /// Packed RGB 3:3:2, 8bpp, (msb)3R 3G 2B(lsb)
        /// Corresponds to `AV_PIX_FMT_RGB8`.
        Rgb8 = AV_PIX_FMT_RGB8 as _,

        /// Packed RGB 1:2:1 bitstream, 4bpp, (msb)1R 2G 1B(lsb)
        /// Corresponds to `AV_PIX_FMT_RGB4`.
        Rgb4 = AV_PIX_FMT_RGB4 as _,

        /// Packed RGB 1:2:1, 8bpp, (msb)1R 2G 1B(lsb)
        /// Corresponds to `AV_PIX_FMT_RGB4_BYTE`.
        Rgb4Byte = AV_PIX_FMT_RGB4_BYTE as _,

        /// Planar YUV 4:2:0, 12bpp, 1 plane for Y and 1 plane for UV components
        /// Corresponds to `AV_PIX_FMT_NV12`.
        Nv12 = AV_PIX_FMT_NV12 as _,

        /// As NV12, but U and V bytes are swapped
        /// Corresponds to `AV_PIX_FMT_NV21`.
        Nv21 = AV_PIX_FMT_NV21 as _,

        /// Packed ARGB 8:8:8:8, 32bpp, ARGBARGB...
        /// Corresponds to `AV_PIX_FMT_ARGB`.
        Argb = AV_PIX_FMT_ARGB as _,

        /// Packed RGBA 8:8:8:8, 32bpp, RGBARGBA...
        /// Corresponds to `AV_PIX_FMT_RGBA`.
        Rgba = AV_PIX_FMT_RGBA as _,

        /// Packed ABGR 8:8:8:8, 32bpp, ABGRABGR...
        /// Corresponds to `AV_PIX_FMT_ABGR`.
        Abgr = AV_PIX_FMT_ABGR as _,

        /// Packed BGRA 8:8:8:8, 32bpp, BGRABGRA...
        /// Corresponds to `AV_PIX_FMT_BGRA`.
        Bgra = AV_PIX_FMT_BGRA as _,

        /// Y, 16bpp, big-endian
        /// Corresponds to `AV_PIX_FMT_GRAY16BE`.
        Gray16Be = AV_PIX_FMT_GRAY16BE as _,

        /// Y, 16bpp, little-endian
        /// Corresponds to `AV_PIX_FMT_GRAY16LE`.
        Gray16Le = AV_PIX_FMT_GRAY16LE as _,

        /// Planar YUV 4:4:0 (1 Cr & Cb sample per 1x2 Y samples)
        /// Corresponds to `AV_PIX_FMT_YUV440P`.
        Yuv440p = AV_PIX_FMT_YUV440P as _,

        /// Planar YUV 4:4:0 full scale (JPEG), deprecated
        /// Corresponds to `AV_PIX_FMT_YUVJ440P`.
        Yuvj440p = AV_PIX_FMT_YUVJ440P as _,

        /// Planar YUV 4:2:0, 20bpp, (1 Cr & Cb sample per 2x2 Y & A samples)
        /// Corresponds to `AV_PIX_FMT_YUVA420P`.
        Yuva420p = AV_PIX_FMT_YUVA420P as _,

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

        /// Planar GBR format, 4:4:4 subsampling.
        /// Corresponds to `AV_PIX_FMT_GBRP`.
        Gbrp = AV_PIX_FMT_GBRP as _,

        /// HW decoding through DXVA2
        /// Corresponds to `AV_PIX_FMT_DXVA2_VLD`.
        Dxva2Vld = AV_PIX_FMT_DXVA2_VLD as _,

        /// Packed RGB 4:4:4, 16bpp, little-endian
        /// Corresponds to `AV_PIX_FMT_RGB444LE`.
        Rgb444Le = AV_PIX_FMT_RGB444LE as _,

        /// Packed RGB 4:4:4, 16bpp, big-endian
        /// Corresponds to `AV_PIX_FMT_RGB444BE`.
        Rgb444Be = AV_PIX_FMT_RGB444BE as _,

        /// Packed BGR 4:4:4, 16bpp, little-endian
        /// Corresponds to `AV_PIX_FMT_BGR444LE`.
        Bgr444Le = AV_PIX_FMT_BGR444LE as _,

        /// Packed BGR 4:4:4, 16bpp, big-endian
        /// Corresponds to `AV_PIX_FMT_BGR444BE`.
        Bgr444Be = AV_PIX_FMT_BGR444BE as _,

        /// 8 bits gray, 8 bits alpha
        /// Corresponds to `AV_PIX_FMT_YA8`.
        Ya8 = AV_PIX_FMT_YA8 as _,

        /// Packed RGB 16:16:16, 48bpp, 16B, 16G, 16R, big-endian
        /// Corresponds to `AV_PIX_FMT_BGR48BE`.
        Bgr48Be = AV_PIX_FMT_BGR48BE as _,

        /// Packed RGB 16:16:16, 48bpp, 16B, 16G, 16R, little-endian
        /// Corresponds to `AV_PIX_FMT_BGR48LE`.
        Bgr48Le = AV_PIX_FMT_BGR48LE as _,

        /// HW acceleration through VDPAU
        /// Corresponds to `AV_PIX_FMT_VDPAU`.
        Vdpau = AV_PIX_FMT_VDPAU as _,

        /// Packed YUV 4:2:2, 16bpp, Y0 Cr Y1 Cb
        /// Corresponds to `AV_PIX_FMT_YVYU422`.
        Yvyu422 = AV_PIX_FMT_YVYU422 as _,

        /// 16 bits gray, 16 bits alpha (big-endian)
        /// Corresponds to `AV_PIX_FMT_YA16BE`.
        Ya16Be = AV_PIX_FMT_YA16BE as _,

        /// 16 bits gray, 16 bits alpha (little-endian)
        /// Corresponds to `AV_PIX_FMT_YA16LE`.
        Ya16Le = AV_PIX_FMT_YA16LE as _,

        /// Planar GBRA 4:4:4:4 32bpp
        /// Corresponds to `AV_PIX_FMT_GBRAP`.
        Gbrap = AV_PIX_FMT_GBRAP as _,

        /// Planar GBRA 4:4:4:4 64bpp, big-endian
        /// Corresponds to `AV_PIX_FMT_GBRAP16BE`.
        Gbrap16Be = AV_PIX_FMT_GBRAP16BE as _,

        /// Planar GBRA 4:4:4:4 64bpp, little-endian
        /// Corresponds to `AV_PIX_FMT_GBRAP16LE`.
        Gbrap16Le = AV_PIX_FMT_GBRAP16LE as _,

        /// HW acceleration through QSV
        /// Corresponds to `AV_PIX_FMT_QSV`.
        Qsv = AV_PIX_FMT_QSV as _,

        /// HW acceleration through Direct3D11
        /// Corresponds to `AV_PIX_FMT_D3D11VA_VLD`.
        D3d11vaVld = AV_PIX_FMT_D3D11VA_VLD as _,

        /// HW acceleration through CUDA
        /// Corresponds to `AV_PIX_FMT_CUDA`.
        Cuda = AV_PIX_FMT_CUDA as _,

        /// Packed RGB 8:8:8, 32bpp, XRGBXRGB... X=unused
        /// Corresponds to `AV_PIX_FMT_0RGB`.
        ZeroRgb = AV_PIX_FMT_0RGB as _,

        /// Packed RGB 8:8:8, 32bpp, RGBXRGBX... X=unused
        /// Corresponds to `AV_PIX_FMT_RGB0`.
        Rgb0 = AV_PIX_FMT_RGB0 as _,

        /// Packed BGR 8:8:8, 32bpp, XBGRXBGR... X=unused
        /// Corresponds to `AV_PIX_FMT_0BGR`.
        ZeroBgr = AV_PIX_FMT_0BGR as _,

        /// Packed BGR 8:8:8, 32bpp, BGRXBGRX... X=unused
        /// Corresponds to `AV_PIX_FMT_BGR0`.
        Bgr0 = AV_PIX_FMT_BGR0 as _,

        /// Planar YUV 4:2:2 24bpp, (1 Cr & Cb sample per 2x1 Y & A samples)
        /// Corresponds to `AV_PIX_FMT_YUVA422P`.
        Yuva422p = AV_PIX_FMT_YUVA422P as _,

        /// Planar YUV 4:4:4 32bpp, (1 Cr & Cb sample per 1x1 Y & A samples)
        /// Corresponds to `AV_PIX_FMT_YUVA444P`.
        Yuva444p = AV_PIX_FMT_YUVA444P as _,

        /// Interleaved chroma YUV 4:2:2, 16bpp
        /// Corresponds to `AV_PIX_FMT_NV16`.
        Nv16 = AV_PIX_FMT_NV16 as _,

        /// Like NV12, with 10bpp per component, little-endian
        /// Corresponds to `AV_PIX_FMT_P010LE`.
        P010Le = AV_PIX_FMT_P010LE as _,

        /// Like NV12, with 10bpp per component, big-endian
        /// Corresponds to `AV_PIX_FMT_P010BE`.
        P010Be = AV_PIX_FMT_P010BE as _,

        /// Hardware decoding through Videotoolbox
        /// Corresponds to `AV_PIX_FMT_VIDEOTOOLBOX`.
        Videotoolbox = AV_PIX_FMT_VIDEOTOOLBOX as _,

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
