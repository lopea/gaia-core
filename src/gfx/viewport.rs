use std::sync::{Arc, Mutex};

use ash::{vk::{SwapchainKHR, SurfaceKHR, Extent2D, SurfaceTransformFlagsKHR, SharingMode, ImageUsageFlags, CompositeAlphaFlagsKHR, SwapchainCreateInfoKHR, PresentModeKHR}, extensions::khr::{Swapchain,Surface}};
use winit::{event_loop::{EventLoop, EventLoopWindowTarget, ControlFlow}, window::{Window, WindowBuilder}, dpi::LogicalSize, platform::run_return::EventLoopExtRunReturn, event::{WindowEvent, Event}};

use crate::gfx::render_system::RenderInstance;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::images::swapchain::SwapchainImage;

pub struct Viewport {
    
    ///reference to the instance for all your rendering needs
    render_inst: Arc<RenderInstance>,

    ///reference to the window 
    win_handle: Arc<Window>,

    surface : Arc<SurfaceKHR>, 

    surface_loader : Arc<Surface>,
    
    ///reference to all the images used for rendering
    swapchain: Arc<Mutex<SwapchainKHR>>,

    swapchain_images: Vec<Arc<SwapchainImage>>,
    ///loads the proper vales from vulkan
    swapchain_loader: Arc<Swapchain>, 
    
    update_swapchain: bool,
}

impl Viewport {
    
    fn cleanup_swapchain(&mut self)
    {
        unsafe{
            self.swapchain_loader.destroy_swapchain(*self.swapchain.lock().unwrap(), None);
        }
    }

    fn create_swapchain(surface: Arc<SurfaceKHR>, 
                        surface_loader: Arc<Surface>,
                        render_inst: Arc<RenderInstance>,
                        swapchain_loader: Arc<Swapchain>) -> (SwapchainKHR, Vec<Arc<SwapchainImage>>){


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
        
        println!("what");
        let swapchain =  unsafe{
            swapchain_loader.create_swapchain(&swapchain_ci, None)
        }.unwrap();

       let images : Vec<Arc<SwapchainImage>> =  unsafe {
            let images_raw = swapchain_loader.get_swapchain_images(swapchain).expect("Cannot get swapchain images!");
            images_raw.iter().map(|image| Arc::new(SwapchainImage::new(image))).collect()
        };

       (swapchain, images)
    }
    
    fn recreate_swapchain(&mut self) {
        self.cleanup_swapchain();
        let (newChain, newImg) = Viewport::create_swapchain(self.surface.clone(),
                                                            self.surface_loader.clone(), 
                                                            self.render_inst.clone(),
                                                            self.swapchain_loader.clone());
        let mut chain_data = self.swapchain.lock().unwrap();
        *chain_data = newChain;
        self.swapchain_images = newImg;
        self.update_swapchain = false;
    }

    fn update(&mut self, event : Event<()>, _window_target: &EventLoopWindowTarget<()>, control_flow:&mut ControlFlow){

        control_flow.set_poll();
        
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            },
            Event::MainEventsCleared =>
            {
               if self.update_swapchain {
                   self.recreate_swapchain();
               }

            },
            Event::WindowEvent { 
                event: WindowEvent::Resized(_),
                ..
            } => {
                self.update_swapchain = true;
            }
            _ => (),
        }
    }

    pub fn run(&mut self, event_loop: &mut EventLoop<()>) {
        event_loop.run_return(|event, target, flow| self.update(event, target, flow));
    }
    
    pub fn new(event_loop: &EventLoop<()>,render_inst: Arc<RenderInstance>, initial_width: u32, initial_height: u32)  -> Self {



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


        let swapchain_loader = Arc::new(Swapchain::new(&render_inst.inst(),
         &render_inst.dev()));
        
        let (swapchain, images) = Viewport::create_swapchain(surface.clone(),
                                  surface_loader.clone(),
                                  render_inst.clone(), 
                                  swapchain_loader.clone()); 

        //create the viewport and send it off!
        Self { 
            render_inst, 
            win_handle,
            surface,
            surface_loader,
            swapchain: Arc::new(Mutex::new(swapchain)),
            swapchain_images: images,
            swapchain_loader,
            update_swapchain: false
        }
    }
    
    fn swapchain(&self) -> Arc<Mutex<SwapchainKHR>> {
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
             self.cleanup_swapchain();

            //destroy the surface
            self.surface_loader.destroy_surface(*self.surface, None);
         }
     }
}
