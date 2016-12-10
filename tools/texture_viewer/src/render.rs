use std::sync::mpsc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::fs::File;
use std::io::BufReader;

use glfw::{self, Context};
use image::{self, DynamicImage, GenericImage};
use capnp;

use common::error::*;

use backend::gl::*;
use backend::gl::types::*;
use backend::gl::bindings as glb;

use combustion_protocols::protocols;

use screen::ScreenQuad;

pub enum RenderSignal {
    Stop,
    Refresh,
    Resize(i32, i32),
    ChangeTexture(PathBuf)
}

pub fn start(mut context: glfw::RenderContext, rx: mpsc::Receiver<RenderSignal>) -> GLResult<()> {
    let mut active_texture = try!(GLTexture::new(GLTextureKind::Texture2D));

    try!(active_texture.set_filter(GLTextureFilter::Nearest, None));

    let mut screen = try!(ScreenQuad::new());

    let screen_vertex_shader = try!(GLShader::from_file("shaders/screen.vert", GLShaderVariant::VertexShader));
    let screen_fragment_shader = try!(GLShader::from_file("shaders/screen.frag", GLShaderVariant::FragmentShader));

    let screen_shader = GLShaderProgramBuilder::new()?
        .attach_shader(screen_vertex_shader)?
        .attach_shader(screen_fragment_shader)?
        .link()?
        .finish();

    let mut resolution: (u32, u32) = (600, 800);
    let mut texture_resolution: (u32, u32) = (0, 0);

    'render: loop {
        let mut viewport_size = None;

        //Block for events then process when necessary
        for event in rx.try_iter() {
            match event {
                RenderSignal::Stop => {
                    break 'render;
                }
                RenderSignal::Refresh => {}
                RenderSignal::Resize(width, height) => {
                    viewport_size = Some((width, height));
                }
                RenderSignal::ChangeTexture(path) => {
                    try!(active_texture.bind());

                    if path.extension().unwrap() == protocols::texture::EXTENSION {
                        use self::protocols::texture;
                        use self::protocols::texture::protocol::{Format as TextureFormat, Compression};
                        use self::protocols::texture::protocol::texture as texture_protocol;

                        let mut file = BufReader::new(File::open(path)?);

                        let texture_message = capnp::serialize_packed::read_message(&mut file, capnp::message::ReaderOptions {
                            traversal_limit_in_words : 64 * 1024 * 1024, nesting_limit : 64
                        }).unwrap();

                        let texture = texture_message.get_root::<texture_protocol::Reader>().unwrap();

                        let width = texture.get_width();
                        let height = texture.get_height();
                        let format = texture.get_format().unwrap();
                        let compression = texture.get_compression().unwrap();
                        let data = texture.get_data().unwrap();

                        let (format, iformat) = match format {
                            TextureFormat::Rgb => (glb::RGB, glb::RGB8),
                            TextureFormat::Rgba => (glb::RGBA, glb::RGBA8),
                            TextureFormat::Luma => (glb::RED, glb::R8),
                            TextureFormat::LumaAlpha => (glb::RG, glb::RG8)
                        };

                        unsafe {
                            glb::TexImage2D(glb::TEXTURE_2D, 0, iformat as GLint,
                                            width as GLsizei, height as GLsizei, 0,
                                            format, glb::UNSIGNED_BYTE, data.as_ptr() as *const _);
                        }

                        try!(active_texture.generate_mipmap());

                        texture_resolution = (width, height);

                        check_errors!();

                    } else {
                        info!("Buffering normal image...");

                        let image = try!(image::open(path));

                        let (width, height) = image.dimensions();

                        let (format, iformat, data) = match image {
                            DynamicImage::ImageLuma8(i) => (glb::RED, glb::R8, i.into_vec()),
                            DynamicImage::ImageLumaA8(i) => (glb::RG, glb::RG8, i.into_vec()),
                            DynamicImage::ImageRgb8(i) => (glb::RGB, glb::RGB8, i.into_vec()),
                            DynamicImage::ImageRgba8(i) => (glb::RGBA, glb::RGBA8, i.into_vec())
                        };

                        unsafe {
                            glb::TexImage2D(glb::TEXTURE_2D, 0, iformat as GLint,
                                            width as GLsizei, height as GLsizei, 0,
                                            format, glb::UNSIGNED_BYTE, data.as_ptr() as *const _);
                        }

                        texture_resolution = (width, height);

                        check_errors!();
                    }
                }
            }
        }

        if let Some((width, height)) = viewport_size {
            unsafe { glb::Viewport(0, 0, width as GLsizei, height as GLsizei); }

            resolution = (width as u32, height as u32);

            check_errors!();

            info!("Viewport resized to {}x{}", width, height);
        }

        try!(screen_shader.use_program());

        info!("Rendering...");
        try!(screen.draw());

        let mut res_uniform = try!(screen_shader.get_uniform("resolution"));
        let mut tex_res_uniform = try!(screen_shader.get_uniform("texture_resolution"));

        try!(res_uniform.float2(resolution.0 as f32, resolution.1 as f32));
        try!(tex_res_uniform.float2(texture_resolution.0 as f32, texture_resolution.1 as f32));

        context.swap_buffers();

        info!("Parking...");
        ::std::thread::park();
        info!("Resuming...");
    }

    Ok(())
}