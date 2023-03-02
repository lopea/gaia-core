use ash::{extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain, Win32Surface, XcbSurface, WaylandSurface},
}, vk::{EntryFnV1_3, InstanceCreateInfo, ApplicationInfo, make_api_version, PhysicalDeviceType}};
use ash::{vk, Entry};
pub use ash::{Device, Instance};
use std::{os::raw::c_char, ffi::CStr};


pub struct RenderManager
{
    
    entry: Entry,
    
    //vulkan instantiation for gaia
    instance: Instance,

    //logical device that allows gaia to call to a GPU
    device: Device,

}


impl RenderManager {
    fn get_instance_ext() -> Vec<*const c_char>
    {
        vec![
           Surface::name().as_ptr(),
           #[cfg(target_os = "windows")]
           Win32Surface::name().as_ptr(),
           #[cfg(target_os = "linux")]
           {
                XcbSurface::name().as_ptr()
           },
           
        ]
    }

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

    fn find_pys_device(inst : &Instance) {
       unsafe{
           let devices = inst.enumerate_physical_devices().expect("No GPU Devices Found!");
           let devProp : Vec<(&vk::PhysicalDevice, vk::PhysicalDeviceProperties)>= devices.iter()
               .map(|x| {
                    (x, inst.get_physical_device_properties(*x))
               }).collect();

           devProp.sort_by_key(|(dev, prop)| {
               match prop.device_type {
                PhysicalDeviceType::DISCRETE_GPU => 0,
                PhysicalDeviceType::INTEGRATED_GPU => 1,
                PhysicalDeviceType::VIRTUAL_GPU => 2,
                PhysicalDeviceType::CPU =>3, 
                PhysicalDeviceType::OTHER => 4,
               }
           });
           devProp
               .iter()
               .find_map(|(dev, prop)|)
           
           
       }
    }

    pub fn new() -> Self { 
        
        let entry = Entry::linked();
        
        
        
        //data for creating instance
        let extensions = RenderManager::get_instance_ext();
        let app_info = RenderManager::get_application_info();
        let layer_names = RenderManager::get_layer_names();


        //collect data to struct
        let instance_ci = InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extensions);

        //create instance
        let instance = unsafe {
            entry.create_instance(&instance_ci, None)
        }.unwrap();

        let surface_loader = Surface::new(&entry, &instance);

        let chosen_phys = RenderManager::find_pys_device(&instance);

        
        Self { entry, instance, } 

    }
    
    pub fn dev(&self) -> &Device {
        &self.device
    }


}

impl Drop for RenderManager {
    fn drop(&mut self) {
        unsafe{
            self.instance.destroy_instance(None);
        }
    }
}
