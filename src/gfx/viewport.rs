use std::sync::Arc;

use ash::{vk::{SwapchainKHR, SurfaceKHR, Extent2D, SurfaceTransformFlagsKHR, SharingMode, ImageUsageFlags, CompositeAlphaFlagsKHR, SwapchainCreateInfoKHR, PresentModeKHR}, extensions::khr::{Swapchain,Surface}};
use winit::{event_loop::EventLoop, window::{Window, WindowBuilder}, dpi::LogicalSize};

use crate::gfx::render_system::RenderInstance;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub struct Viewport {
    
    ///reference to the instance for all your rendering needs
    render_inst: Arc<RenderInstance>,

    ///reference to the window 
    win_handle: Arc<Window>,

    surface : Arc<SurfaceKHR>, 

    surface_loader : Arc<Surface>,
    
    ///reference to all the images used for rendering
    swapchain: Arc<SwapchainKHR>,
    
    ///loads the proper values from vulkan
    swapchain_loader: Arc<Swapchain>, 
}

impl Viewport {


    pub fn new(event_loop: &EventLoop<()>,render_inst: Arc<RenderInstance>, initial_width: u32, initial_height: u32)  -> Arc<Viewport> {



        let win_handle = Arc::new(WindowBuilder::new()
            .with_inner_size(LogicalSize::new(initial_width, initial_height))
            .build(event_loop).unwrap());
        

        let surface = Arc::new( unsafe{
            ash_window::create_surface(&render_inst.entry(), &render_inst.inst(), 
                                       win_handle.raw_display_handle(), 
                                       win_handle.raw_window_handle(), 
                                       None) 
        }.expect("Unable to create Surface!"));

        let surface_loader = Arc::new(Surface::new(&render_inst.entry(), &render_inst.inst()));
        
        let present_types =unsafe { 
            surface_loader.get_physical_device_surface_present_modes(*render_inst.physical_dev(),

                                                                    *surface)
        }.expect("No present types found!"); 

        let surface_format = unsafe {
            surface_loader.get_physical_device_surface_formats(*render_inst.physical_dev(), 
                                                               *surface)
        }.unwrap()[0];


        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(*render_inst.physical_dev(),
            *surface)
        }.unwrap();

        let mut img_count = capabilities.min_image_count + 1;
        
        //WARN: For whatever reason my drivers have the min image count greater than the max????
        // i cant clamp max value without the specification yelling at me
        let max_img = capabilities.min_image_count.max(capabilities.max_image_count);
        
        if img_count > max_img {
            img_count = max_img;
        }
        
        

        let surface_resolution = match capabilities.current_extent.width {
            std::u32::MAX => Extent2D {
                width: 1024, 
                height: 768,
            },
            _ => capabilities.current_extent,
        };

        let pre_transform = if capabilities.supported_transforms.contains(SurfaceTransformFlagsKHR::IDENTITY) {
            SurfaceTransformFlagsKHR::IDENTITY
        } else {
            capabilities.current_transform
        };


        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(*render_inst.physical_dev(),
            *surface)
        }.unwrap();

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| {
                mode == PresentModeKHR::MAILBOX
            })
            .unwrap_or(PresentModeKHR::FIFO);

        let swapchain_loader = Arc::new(Swapchain::new(&render_inst.inst(),
         &render_inst.dev()));
        
        let swapchain_ci = SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(img_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .build();
        
        let swapchain = Arc::new( unsafe{
            swapchain_loader.create_swapchain(&swapchain_ci, None)
        }.unwrap());

        //create the viewport and send it off!
        Arc::new(Self { 
            render_inst, 
            win_handle,
            surface,
            surface_loader,
            swapchain, 
            swapchain_loader 
        })
    }
    
    fn swapchain(&self) -> Arc<SwapchainKHR> {
         self.swapchain.clone()
    }

    fn surface(&self) -> Arc<SurfaceKHR> {
        self.surface.clone()
    }
    
}

impl Drop for Viewport {
     fn drop(&mut self) {
         unsafe{
             //destroy the swapchain
            self.swapchain_loader.destroy_swapchain(*self.swapchain,None);

            //destroy the surface
            self.surface_loader.destroy_surface(*self.surface, None);
         }
     }
}
