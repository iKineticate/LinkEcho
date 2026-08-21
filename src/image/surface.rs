use windows::Win32::{
    D2D_COLOR_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BITMAP_PROPERTIES1,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_PIXEL_FORMAT,
    D2D1CreateFactory, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice,
    DXGI_FORMAT_R8G8B8A8_UNORM, HMODULE, ID2D1Device, ID2D1DeviceContext, ID2D1Factory1,
    ID3D11Device, IDXGIDevice,
};
use windows::core::Interface;
use windows_numerics::Matrix3x2;
use windows_reactor::SurfaceImageSource;

pub struct ImageSurfaceContext {
    d2d_device: ID2D1Device,
}

impl ImageSurfaceContext {
    pub fn new() -> windows::core::Result<Self> {
        let mut d3d_device: Option<ID3D11Device> = None;
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT as u32,
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION as u32,
                Some(&mut d3d_device),
                None,
                None,
            )
            .ok()?;
        }
        let d3d_device = d3d_device.unwrap();

        let d2d_factory: ID2D1Factory1 =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        let dxgi_device: IDXGIDevice = d3d_device.cast()?;
        let d2d_device = unsafe { d2d_factory.CreateDevice(&dxgi_device)? };

        Ok(Self { d2d_device })
    }

    pub fn build_surface_from_buffer(
        &self,
        width: u32,
        height: u32,
        rgba_date: &[u8],
    ) -> windows::core::Result<SurfaceImageSource> {
        let surface = SurfaceImageSource::new(width as i32, height as i32)?;
        surface.set_device(&self.d2d_device)?;

        let (context, (offset_x, offset_y)) =
            surface.begin_draw::<ID2D1DeviceContext>(0, 0, width as i32, height as i32)?;

        unsafe {
            let mut dpi_x = 0.0f32;
            let mut dpi_y = 0.0f32;
            context.GetDpi(&mut dpi_x, &mut dpi_y);

            context.SetTransform(&Matrix3x2::translation(offset_x as f32, offset_y as f32));

            let clear_color = D2D_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            };
            context.Clear(Some(&clear_color));

            let size = D2D_SIZE_U { width, height };
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED, // 预乘模式
                },
                dpiX: dpi_x,
                dpiY: dpi_y,
                ..Default::default()
            };

            let pixels = rgba_date.as_ptr();

            let bitmap = context.CreateBitmap(size, Some(pixels.cast()), width * 4, &props)?;

            context.DrawBitmap(
                &bitmap,
                None,
                1.0,
                D2D1_INTERPOLATION_MODE_LINEAR,
                None,
                None,
            );
        }

        surface.end_draw()?;

        Ok(surface)
    }
}

// fn premultiply_rgba(rgba: &[u8]) -> Vec<u8> {
//     let mut out = vec![0u8; rgba.len()];

//     for (src, dst) in rgba.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
//         let a = src[3];

//         if a == 255 {
//             dst.copy_from_slice(src);
//             continue;
//         }

//         if a == 0 {
//             continue;
//         }

//         let a = a as u32;
//         dst[0] = (((src[0] as u32 * a + 128) * 257) >> 16) as u8;
//         dst[1] = (((src[1] as u32 * a + 128) * 257) >> 16) as u8;
//         dst[2] = (((src[2] as u32 * a + 128) * 257) >> 16) as u8;
//         dst[3] = a as u8;
//     }

//     out
// }