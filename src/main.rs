use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
    },
    image::{view::ImageView, ImageAccess, ImageUsage, SwapchainImage, self},
    impl_vertex,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    single_pass_renderpass,
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
    Version
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

fn on_resize(
    images : &[Arc<SwapchainImage>],
    render_pass : Arc<RenderPass>,
    viewport: &mut Viewport
    ) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[0] as f32];
    
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo { 
                    attachments: vec![view],
                    ..Default::default()
                }
                ).unwrap()
        }).collect::<Vec<_>>()
}

fn main() {
    //get vulkan library
    let library = VulkanLibrary::new().expect("No Vulkan Library Found!");
    let instance_ext = InstanceExtensions{
        khr_surface: true,
        khr_win32_surface: cfg!(windows),
        khr_xcb_surface: cfg!(unix),
        ..Default::default()
    };

    //create vulkan instance 
    let _instance = Instance::new(library, InstanceCreateInfo { 
        application_name: Some(String::from("Gaia Test")),
        application_version: Version::V1_6, 
        enabled_extensions: instance_ext,
        engine_name: Some(String::from("Gaia Core")),
        ..Default::default() 
    }).unwrap_or_else(|err| panic!("Cannot create vulkan instance! err: {:?}", err));

    //get device ext
    let device_ext = DeviceExtensions {
        khr_swapchain: true, 
        ..DeviceExtensions::empty()
    };
    

    //create a new window loop
    let event_loop = EventLoop::new();
    //create a new window with loop
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, _instance.clone())
        .expect("Unable to create surface!");
    
    println!("Available devices:");
    for i in _instance.enumerate_physical_devices().unwrap(){
       println!("{}",i.properties().device_name);
       println!("\tDevice Type: {:#?}", i.properties().device_type);
    }
    
    //get physical device and queues    
    let (physical_device,queue_index) = _instance
        
        //gets the physical device
        .enumerate_physical_devices()
        //unwraps it, if it does have any devices, throw panic with msg
        .expect("No Physical Device found!")
        //get all the devices that support the extensions that we want
        .filter(|x| 
            x.supported_extensions().contains(&device_ext)
        )
        //create a map of the current queue index and the current device, 
        //at the same time, filter out any devices that dont support the graphics queue
        .filter_map(|x| {
            //get all the queue values of this physical device,
            x.queue_family_properties()
                //convert a "stream" into a set of iterators
                .iter()
                //convert the iterators into a map of (index, iter)
                .enumerate()
                //go through all values and search for the ones that support surfaces and graphics
                //queues
                .position(|(i, q)| 
                    q.queue_flags.graphics
                    && x.surface_support(i as u32, &surface).unwrap_or(false) 
                )
                //convert the setup into a vector of tuples with the graphics queue included
                .map(|i| (x, i as u32))
        })
        //sort the devices by level of importance
        .min_by_key(|(x, _)|{
            match x.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            }
        })
        .expect("No physical devices found!");
    
    //show the device info
    println!("Using Device: {} \n\tType:{:#?}", physical_device.properties().device_name, 
             physical_device.properties().device_type);

    
    let (device, mut queues) = Device::new(physical_device, DeviceCreateInfo { 
        enabled_extensions: device_ext,  
        queue_create_infos: vec![QueueCreateInfo{
            queue_family_index: queue_index,
            ..Default::default()
        }],
        ..Default::default()
    }).unwrap();
    
    let _queue = queues.next().unwrap();

    let (mut _swapchain, _images)= {
        let sur_cap = device
            .physical_device()
            .surface_capabilities(&surface, Default::default()).unwrap();

        let img_fmt = Some(
            device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
        );

        let win = surface.object().unwrap().downcast_ref::<Window>().unwrap();
    
        let img_cnt = (sur_cap.min_image_count + 1).clamp(0, sur_cap.max_image_count.unwrap_or(u32::MAX));

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo { 
                min_image_count: img_cnt,
                image_format: img_fmt,
                image_extent: win.inner_size().into(),image_usage: ImageUsage
                {
                    color_attachment: true,
                    ..Default::default()
                },
                composite_alpha: sur_cap.supported_composite_alpha.iter().next().unwrap(),
                ..Default::default()
            }
            ).unwrap()
    };

    //create allocator for GPU
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    //setup vertex
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
    struct Vert{
        position: [f32; 2],
        color: [f32; 3],
    }

    impl_vertex!(Vert, position,  color);

    //create vertex buffer
    let mesh = [
        Vert{
            position: [-0.5, -0.25],
            color: [0.0,0.0,1.0]
        },
        Vert{
            position: [0.0, 0.75],
            color: [1.0,0.0,0.0]
        },
        Vert{
            position: [0.25, -0.1],
            color: [0.0,1.0,0.0]
        },

    ];

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage{
            vertex_buffer: true,
            ..BufferUsage::default()
        },
        false,
        mesh
    ).unwrap();
    

    mod v_shader {

        vulkano_shaders::shader!{
            ty:"vertex",
            src: "
                #version 450
                layout(location = 0) in vec2 position;
                layout(location = 1) in vec3 color;
                layout(location = 0)out vec4 outColor;
                void main(){
                    gl_Position = vec4(position, 0.0, 1.0);
                    gl_Position.y = -gl_Position.y;
                    outColor = vec4(color, 1.0);
                }
            "
        }
    }
    mod f_shader {
        vulkano_shaders::shader! {
            ty: "fragment",
            src:"
                #version 450
                layout(location = 0) in vec4 outColor;
                layout(location = 0) out vec4 FragColor;

                void main() {
                    FragColor = outColor;    
                }
            "
        }
    }

    let vs = v_shader::load(device.clone()).unwrap();
    let fs = f_shader::load(device.clone()).unwrap();

    let render_pass = single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: _swapchain.image_format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
        ).unwrap();

    let pipe =  GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .vertex_input_state(BuffersDefinition::new().vertex::<Vert>())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .build(device.clone())
        .unwrap();


    let mut viewport = Viewport
    {
        origin: [0.0,0.0],
        dimensions: [0.0,0.0],
        depth_range: 0.0..1.0,
    };

    
    let mut framebuffers = on_resize(&_images, render_pass.clone(), &mut viewport);
    println!("So far So Good!");
    
    let command_buffer_alloc = StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let mut recreate_swapchain = false;

    let mut prev_frame_end = Some(sync::now(device.clone()).boxed());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            Event::WindowEvent { 
                event: WindowEvent::Resized(_),
                ..
            } => {
              recreate_swapchain = true;
            },
            Event::RedrawEventsCleared => {
                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
                let dimensions = window.inner_size();
                if dimensions.width == 0 || dimensions.height == 0 {
                    return;
                }
                prev_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {

                    let (new_swapchain, new_images) = match _swapchain.recreate(SwapchainCreateInfo{
                        image_extent: dimensions.into(),
                        .._swapchain.create_info()
                    }) {
                        Ok(r) => r,
                        Err(vulkano::swapchain::SwapchainCreationError::ImageExtentNotSupported{..}) => return,
                        Err(e) => panic!("Cannot recreate Swapchain! {e:?}"),
                    };
                    
                    _swapchain = new_swapchain;
                    framebuffers = on_resize(&new_images, render_pass.clone(), &mut viewport);
                    recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) = 
                    match acquire_next_image(_swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        },
                        Err(e) => panic!("failed to get next image! Err: {e:?}"),
                    };
                if suboptimal {
                    recreate_swapchain = true;
                }
                
                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_alloc,
                    _queue.queue_family_index(),
                    vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit)
                    .unwrap();
                
                builder.begin_render_pass(RenderPassBeginInfo { 
                    clear_values: vec![Some([0.0,0.0,0.0,1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffers[image_index as usize].clone())
                },
                vulkano::command_buffer::SubpassContents::Inline)
                    .unwrap()
                .set_viewport(0, [viewport.clone()])
                .bind_pipeline_graphics(pipe.clone())
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .draw(vertex_buffer.len() as u32, 1,0,0)
                .unwrap()
                .end_render_pass()
                .unwrap();

                let command_buff = builder.build().unwrap();

                let future = prev_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(_queue.clone(), command_buff)
                    .unwrap()
                    .then_swapchain_present(_queue.clone(), 
                                            SwapchainPresentInfo::swapchain_image_index(_swapchain.clone(), image_index))
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        prev_frame_end = Some(future.boxed());
                    },
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        prev_frame_end = Some(sync::now(device.clone()).boxed());
                    },
                    Err(e) => {
                        panic!("Failed to flush future! Err: {e:?}");
                    }
                }
            }
            _ =>()

        }
    })
}
