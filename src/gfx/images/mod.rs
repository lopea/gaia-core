pub mod swapchain;
use std::sync::Arc;
use ash::vk;


pub trait Image {
    fn handle(&self) -> Arc<vk::Image>;
}
