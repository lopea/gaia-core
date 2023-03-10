use ash::{extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain, Win32Surface, XcbSurface, WaylandSurface},
}, vk::{EntryFnV1_3, InstanceCreateInfo, ApplicationInfo, make_api_version, PhysicalDeviceType, SurfaceKHR, PhysicalDevice, PhysicalDeviceProperties, DeviceCreateInfo, DeviceQueueCreateInfo, SwapchainKHR, Queue}};
use ash::{vk, Entry};
pub use ash::{Device, Instance};
use winit::{window::{Window, WindowBuilder}, dpi::{Size, LogicalSize, LogicalPosition}, event_loop::EventLoop};
use std::{os::raw::c_char, ffi::CStr, sync::Arc, mem::transmute};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub struct RenderInstance
{
    //entry point for vulkan     
    entry: Entry,
    
    //vulkan instantiation for gaia
    instance: Instance,

    //logical device that allows gaia to call to a GPU
    logical_device: Device,

    physical_device: PhysicalDevice,

    queue_family_index : u32,

    present_queue : Queue, 
       
}


impl RenderInstance {
    fn get_layer_names() -> Vec<*const c_char>
    {
        unsafe {
            let layer_names = [CStr::from_bytes_with_nul_unchecked(
                    b"VK_LAYER_KHRONOS_validation\0",
            )];
            layer_names.iter().map(|name| name.as_ptr()).collect()
        }

    }

    fn get_application_info() -> ApplicationInfo {
        unsafe{
            ApplicationInfo::builder()
                .application_name(CStr::from_bytes_with_nul_unchecked(b"Gaia Core Engine"))
                .engine_name(CStr::from_bytes_with_nul_unchecked(b"Gaia Core Engine"))
                .application_version(0)
                .engine_version(1)
                .api_version(make_api_version(1,3, 0, 0))
                .build()
        }
    }

    fn find_pys_device(inst : &Instance, surf: &SurfaceKHR, ent: &Entry, surface_loader : &Surface) -> (PhysicalDevice, u32, PhysicalDeviceProperties){

       unsafe{
           let devices = inst.enumerate_physical_devices().expect("No GPU Devices Found!");
           let mut devProp : Vec<(&vk::PhysicalDevice, vk::PhysicalDeviceProperties)>= devices.iter()
               .map(|x| {
                    (x, inst.get_physical_device_properties(*x))
               }).collect();

           devProp.sort_by_key(|(_, prop)| {
               match prop.device_type {
                PhysicalDeviceType::DISCRETE_GPU => 0,
                PhysicalDeviceType::INTEGRATED_GPU => 1,
                PhysicalDeviceType::VIRTUAL_GPU => 2,
                PhysicalDeviceType::CPU =>3, 
                PhysicalDeviceType::OTHER => 4,
                _ => 5,
               }
           });
            devProp
               .iter()
               .find_map(|(dev, prop)| {
                   inst.get_physical_device_queue_family_properties(**dev)
                       .iter()
                       .enumerate()
                       .find_map(|(index, info)|{
                           let supports_graph_and_surface = 
                                  info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                  && surface_loader.get_physical_device_surface_support(**dev, index as u32, *surf).unwrap();


                           if supports_graph_and_surface {
                               Some((**dev, index as u32, *prop))
                           }
                           else {
                                 None
                           }

                       })
               })
           .expect("No devices found...")       
       }
    }
    
    
    pub fn new(event_loop : &EventLoop<()>) -> Arc<Self> { 
        
        let entry = Entry::linked();
        
        let window = WindowBuilder::new()
            .with_title("Gaia Core Engine")
            .with_inner_size(LogicalSize::new(1, 1))
            .with_decorations(false)
            .with_active(false)
            .with_position(LogicalPosition::new(0,0)) 
            .build(event_loop)
            .unwrap();
        
        window.set_transparent(true);
        window.set_minimized(true);
        
        
        //data for creating instance
        let extensions = ash_window::enumerate_required_extensions(window.raw_display_handle())
            .expect("unable to get extensions!");
        let app_info = RenderInstance::get_application_info();
        let layer_names = RenderInstance::get_layer_names();


        //collect data to struct
        let instance_ci = InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extensions);

        //create instance
        let instance = unsafe {
            entry.create_instance(&instance_ci, None)
        }.unwrap();

        //create dummy surface
        //
        let surface = unsafe{
            ash_window::create_surface(&entry, &instance, 
                                       window.raw_display_handle(), 
                                       window.raw_window_handle(), 
                                       None) 
        }.expect("Unable to create Surface!");

        let surface_loader = Surface::new(&entry, &instance); 

        let (physical_device, queue_family_index, devProps) = RenderInstance::find_pys_device(&instance, &surface, &entry, &surface_loader);

        let priority = [1.0f32];

        let queue_ci = DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priority)
            .build();

        let device_extension_names_raw = [
            Swapchain::name().as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            KhrPortabilitySubsetFn::name().as_ptr(),
            ];
        
        let dev_ci = DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_ci])
            .enabled_extension_names(&device_extension_names_raw)
            .build();
        
        let logical_device = unsafe {
            instance.create_device(physical_device, &dev_ci, None).unwrap()
        };

        let present_queue = unsafe{
            logical_device.get_device_queue(queue_family_index, 0)
        };
        
        unsafe{
                surface_loader.destroy_surface(surface, None);
        }
        Arc::new(Self {
            instance,
            entry,
            logical_device,
            physical_device,
            queue_family_index,
            present_queue
        }) 

    }
   
    /// returns the vulkan entry point 
    /// used to query global vulkan properties and instances
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    ///the current device to get in the 
    pub fn dev(&self) -> &Device{
        &self.logical_device
    }

    pub fn inst(&self) -> &Instance {
        &self.instance
    }

    pub fn present_queue(&self) -> &Queue {
        &self.present_queue
    }

    pub fn physical_dev(&self) -> &PhysicalDevice {
        &self.physical_device
    }

}

impl Drop for RenderInstance {
    fn drop(&mut self) {
        unsafe{
            self.logical_device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

