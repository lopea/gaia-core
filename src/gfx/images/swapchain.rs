use std::sync::Arc;

use ash::vk;

use super::Image;


pub struct SwapchainImage 
{
    img_handle : Arc<vk::Image>,
}

impl SwapchainImage {
    pub fn new(img: &vk::Image) -> Self {
        Self {
            img_handle: Arc::new(*img)
        }
    }
}

impl Image for SwapchainImage {
    fn handle(&self) -> std::sync::Arc<ash::vk::Image> {
        self.img_handle.clone()
    }
}

