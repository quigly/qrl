/* TODO

- Convert traits to struct methods
*/

use ash::vk;

use super::*;

struct SwapchainSupportInfo
{
	capabilities: vk::SurfaceCapabilitiesKHR,
	formats: Vec<vk::SurfaceFormatKHR>,
	modes: Vec<vk::PresentModeKHR>
}

#[derive(Clone)]
pub struct VkInstance
{
    pub handle: ash::Instance,
    pub entry: ash::Entry,
    pub debug_utils: ash::extensions::ext::DebugUtils,
    pub utils_messenger: vk::DebugUtilsMessengerEXT,
}

impl VkInstance
{
    pub fn new() -> Result<Self, InstanceError>
    {
        let entry = unsafe { ash::Entry::load() };
        if entry.is_err()
        {
            return Err(InstanceError::ApiNotSupported);
        }
        let entry = entry.unwrap();

        let layer_names: Vec<std::ffi::CString> =
            vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layer_name_pointers: Vec<*const i8> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        let surface_extension_name = std::ffi::CString::new(qpl::vk_get_surface_extension()).unwrap();
        let extension_name_pointers: Vec<*const i8> =
            vec![
                ash::extensions::ext::DebugUtils::name().as_ptr(),
                ash::extensions::khr::Surface::name().as_ptr(),
                surface_extension_name.as_ptr()
            ];

        let application_info = vk::ApplicationInfo::builder()
            .api_version(vk::make_api_version(0, 1, 0, 0))
            .build();
        
        let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT
        {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                //| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        };

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .push_next(&mut debugcreateinfo)
            .application_info(&application_info)
            .enabled_extension_names(&extension_name_pointers)
            .enabled_layer_names(&layer_name_pointers);

        let handle = unsafe { entry.create_instance(&instance_create_info, None) }.unwrap();

        let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &handle);
        let utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(&debugcreateinfo, None).unwrap() };

        println!("Created vulkan instance!");

        Ok(Self
        {
            handle,
            entry,
            debug_utils,
            utils_messenger
        })
    }

	fn select_physical_device(&self) -> Result<VkPhysicalDevice, ()>
	{
		let handles = unsafe { self.handle.enumerate_physical_devices() }.unwrap();

		let mut chosen_device: Option<VkPhysicalDevice> = None;

		for handle in &handles
		{
			let physical_device = VkPhysicalDevice::new(*handle, &self.handle);

			chosen_device = Some(physical_device);
			break;
		}

		match chosen_device
		{
			Some(physical_device) => { Ok(physical_device) },
			None => { Err(()) }
		}
	}
}

impl AbstractInstance for VkInstance
{
    fn as_any(&self) -> &dyn Any { self }

    fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>
    {
        let handle = window.vk_create_surface(&self.entry, &self.handle, None);
        let loader = ash::extensions::khr::Surface::new(&self.entry, &self.handle);

        Ok(Surface { internal: Rc::new(VkSurface { handle, loader }) })
    }

    fn create_device(&self, surface: &Surface) -> Result<Device, ()>
    {
        let physical_device = self.select_physical_device().unwrap();
        let surface = surface.downcast_ref::<VkSurface>().unwrap();
        let queue_family_properties = unsafe { self.handle.get_physical_device_queue_family_properties(physical_device.handle) };

        let queue_family_index: u32 =
        {
            let mut found_index: Option<u32> = None;

            for (index, queue_family) in queue_family_properties.iter().enumerate()
            {
                let mut present_support: bool = unsafe { surface.loader.get_physical_device_surface_support(physical_device.handle, index as _, surface.handle) }.unwrap();

                if queue_family.queue_count > 0 &&
                    queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
                    queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) &&
                    present_support
                {
                    found_index = Some(index as u32);
                }
            }

            found_index.unwrap()
        };

        let priortities = [ 1.0f32 ];
        let queue_infos =
        [
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priortities)
                .build()
        ];
        let layer_names: Vec<std::ffi::CString> =
            vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layer_name_pointers: Vec<*const i8> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        let device_extension_name_pointers: Vec<*const i8> =
            vec![ash::extensions::khr::Swapchain::name().as_ptr()];
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extension_name_pointers)
            .enabled_layer_names(&layer_name_pointers);
        let handle = unsafe { self.handle.create_device(physical_device.handle, &device_create_info, None).unwrap() };

        Ok(Device { internal: Box::new(VkDevice { handle, instance: self.handle, queue_family_index, physical_device }) })
    }

    fn create_swapchain(&self, device: &Device, surface: &Surface, create_info: &SwapchainCreateInfo) -> Result<Swapchain, ()>
    {
        let device: &VkDevice = device.downcast_ref::<VkDevice>().unwrap();
        let surface: &VkSurface = surface.downcast_ref::<VkSurface>().unwrap();

		let swapchain_info = device.get_swapchain_support_info(surface);

        let surface_capabilities = choose_swap_surface_format(&swapchain_info.formats);
        let surface_present_modes = choose_swap_present_mode(&swapchain_info.modes, create_info.present_mode);
        let surface_formats = choose_swap_surface_format(&swapchain_info.formats);

        let queue_families = [ device.queue_family_index ];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.handle)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count)
            )
            .image_format(surface_formats.first().unwrap().format)
            .image_color_space(surface_formats.first().unwrap().color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_families)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);
        let loader = ash::extensions::khr::Swapchain::new(&self.handle, &device.handle);
        let handle = unsafe { loader.create_swapchain(&swapchain_create_info, None).unwrap() };

        let swapchain_images = unsafe { loader.get_swapchain_images(handle).unwrap() };

        let mut images: Vec<VkImage> = Vec::with_capacity(swapchain_images.len());
        let mut views: Vec<VkImageView> = Vec::with_capacity(swapchain_images.len());

        for handle in swapchain_images.iter()
        {
            images.push(VkImage { handle: *handle });

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*handle)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            
            views.push(VkImageView { handle: unsafe { device.handle.create_image_view(&imageview_create_info, None) }.unwrap() });
        }

        Ok(Swapchain { internal: Rc::new(VkSwapchain { handle, loader, images, views }) })
    }


}

#[derive(Clone)]
pub struct VkDevice
{
    pub handle: ash::Device,
	pub instance: ash::Instance,
    pub queue_family_index: u32,
	pub physical_device: VkPhysicalDevice
}

impl VkDevice
{
	fn get_swapchain_support_info(&self, surface: &VkSurface) -> SwapchainSupportInfo
	{
		let capabilities: vk::SurfaceCapabilitiesKHR = unsafe
		{
			surface.loader.get_physical_device_surface_capabilities(self.physical_device.handle, surface.handle)
		}.unwrap();

		let formats: Vec<vk::SurfaceFormatKHR> = unsafe
		{
			surface.loader.get_physical_device_surface_formats(self.physical_device.handle, surface.handle)
		}.unwrap();

		let modes: Vec<vk::PresentModeKHR> = unsafe
		{
			surface.loader.get_physical_device_surface_present_modes(self.physical_device.handle, surface.handle)
		}.unwrap();

		SwapchainSupportInfo { capabilities, formats, modes }
	}
}

impl AbstractDevice for VkDevice
{
    fn as_any(&self) -> &dyn Any { self }

    fn get_device_queue(&self) -> Result<Queue, ()>
    {
        let handle = unsafe { self.handle.get_device_queue(self.queue_family_index, 0) };
        Ok(Queue { internal: Rc::new(VkQueue { handle }) })
    }

	fn get_physical_device_properties(&self) -> Result<PhysicalDeviceProperties, ()>
	{
		Ok(PhysicalDeviceProperties
		{
			vendor_id: self.physical_device.properties.vendor_id,
			device_id: self.physical_device.properties.device_id,
			device_type: match self.physical_device.properties.device_type
			{
				vk::PhysicalDeviceType::CPU => DeviceType::CPU,
				vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGPU,
				vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGPU,
				vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGPU,
				_ => DeviceType::Other
			},
			device_name: unsafe { std::ffi::CStr::from_ptr(self.physical_device.properties.device_name.as_ptr()) }.to_str().unwrap().to_owned()
		})
	}

    fn create_shader_module(&self, create_info: &ShaderModuleCreateInfo) -> Result<ShaderModule, ShaderModuleError>
    {
        let code: Vec<u32> = match &create_info.source
        {
            ShaderModuleSource::Spirv(code) =>
            {
                code.clone()
            },
            _ => { panic!() }
        };

        let handle = unsafe { self.handle.create_shader_module(&ash::vk::ShaderModuleCreateInfo::builder()
            .code(&code)
            .build(), None) }.unwrap();

        Ok(ShaderModule { internal: Rc::new(VkShaderModule { handle, stage: create_info.stage }) })
    }
}

#[derive(Clone)]
pub struct VkPhysicalDevice
{
    pub handle: ash::vk::PhysicalDevice,
    pub supported_features: vk::PhysicalDeviceFeatures,
    pub properties: vk::PhysicalDeviceProperties,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_properties: Vec<vk::QueueFamilyProperties>,
}

impl VkPhysicalDevice
{
    pub fn new(handle: vk::PhysicalDevice, instance: &ash::Instance) -> Self
    {
        let supported_features = unsafe { instance.get_physical_device_features(handle) };
        let properties = unsafe { instance.get_physical_device_properties(handle) };
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(handle) };
        let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(handle) };

        Self
        {
            handle, supported_features, properties, memory_properties, queue_family_properties
        }
    }
}

#[derive(Clone)]
pub struct VkQueue
{
    pub handle: ash::vk::Queue
}

impl AbstractQueue for VkQueue
{
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct VkSurface
{
    pub handle: vk::SurfaceKHR,
    pub loader: ash::extensions::khr::Surface
}

impl AbstractSurface for VkSurface
{
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct VkImage
{
    pub handle: vk::Image
}

impl VkImage
{
    
}

#[derive(Clone)]
pub struct VkImageView
{
    pub handle: vk::ImageView
}

impl VkImageView
{
    
}

#[derive(Clone)]
pub struct VkSwapchain
{
    pub handle: vk::SwapchainKHR,
    pub loader: ash::extensions::khr::Swapchain,
    pub images: Vec<VkImage>,
    pub views: Vec<VkImageView>
}

impl AbstractSwapchain for VkSwapchain
{
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct VkShaderModule
{
    pub handle: vk::ShaderModule,
    pub stage: ShaderStage
}

impl AbstractShaderModule for VkShaderModule
{
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct VkRenderPass
{
    pub handle: vk::RenderPass
}

impl VkRenderPass
{
    
}

#[derive(Clone)]
pub struct VkRenderPipeline
{
    pub handle: vk::Pipeline
}

impl VkRenderPipeline
{
    
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32
{
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

fn choose_swap_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR
{
	for format in formats
	{
		if format.format == vk::Format::B8G8R8A8_SRGB &&
			format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
		{
			return *format;
		}
	}

	formats[0]
}

fn choose_swap_present_mode(modes: &[vk::PresentModeKHR], preferred_mode: PresentMode) -> vk::PresentModeKHR
{
	for mode in modes
	{
		if *mode == vk::PresentModeKHR::MAILBOX && preferred_mode == PresentMode::Mailbox
		{
			return *mode;
		}
		else if *mode == vk::PresentModeKHR::FIFO && preferred_mode == PresentMode::Fifo
		{
			return *mode;
		}
		else if *mode == vk::PresentModeKHR::FIFO_RELAXED && preferred_mode == PresentMode::FifoRelaxed
		{
			return *mode;
		}
		else if *mode == vk::PresentModeKHR::IMMEDIATE && preferred_mode == PresentMode::Immediate
		{
			return *mode;
		}
	}

	vk::PresentModeKHR::FIFO
}

fn choose_swap_extent(window: &qpl::Window, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D
{
	if capabilities.current_extent.width != std::u32::MAX
	{
		return capabilities.current_extent;
	}

	let mut actual_extent = vk::Extent2D { width: window.width, height: window.height };

	actual_extent.width = actual_extent.width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
	actual_extent.height = actual_extent.height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);

	actual_extent
}