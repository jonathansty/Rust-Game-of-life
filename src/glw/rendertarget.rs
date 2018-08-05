extern crate gl;
extern crate std;

use super::color::Color;
use super::math::*;

use gl::types::*;

pub struct RenderTarget{
    size: Vec2<i32>,
    fb: GLuint,
    tex: GLuint,
}

impl Default for RenderTarget{
    fn default() -> RenderTarget
    {
        RenderTarget{
            size: Vec2::<i32>{x: 32, y: 32},
            fb: 0,
            tex: 0
        }
    }
}

impl RenderTarget{

    /// Creates a new render target with a specified size
    pub fn new(size : Vec2<i32>) -> Result<RenderTarget, &'static str> {
        let mut tex = 0;
        let mut fb = 0;

        unsafe {
            gl::GenFramebuffers(1,&mut fb);
            gl::BindFramebuffer(gl::FRAMEBUFFER,fb);

            gl::GenTextures(1,&mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MAG_FILTER,gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MIN_FILTER,gl::NEAREST as i32);

            gl::FramebufferTexture(gl::FRAMEBUFFER,gl::COLOR_ATTACHMENT0, tex,0);

            gl::TexImage2D(gl::TEXTURE_2D,0,gl::RGBA as i32, size.x, size.y, 0, gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null());

            let complete = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D,0);

            if complete != gl::FRAMEBUFFER_COMPLETE {
                return Err("Render target creation failed. The framebuffer was not complete.");
            }
        }

        Ok( RenderTarget{
            size,
            fb,
            tex
        } )

    }

    pub fn map_data(&mut self, data : &Vec<Color> ) {
        unsafe{
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fb);
            gl::BindTexture(gl::TEXTURE_2D, self.tex);

            gl::TexImage2D(gl::TEXTURE_2D,0,gl::RGBA as i32, self.size.x, self.size.y, 0, gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as *const std::os::raw::c_void);

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER,0);
        }
    }

	pub fn get_fb(&self) -> GLuint{
		self.fb
	}

    pub fn get_texture(&self) -> GLuint
    {
        self.tex
    }
}
impl Drop for RenderTarget
{
    fn drop(&mut self){
        unsafe{
            gl::DeleteFramebuffers(1, &self.fb);
            gl::DeleteTextures(1, &self.tex);
        }
    }
}
