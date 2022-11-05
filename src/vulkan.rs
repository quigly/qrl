/* TODO

- Convert traits to struct methods
*/

use ash::vk;

use super::*;

#[derive(Clone)]
pub struct VkInstance
{
    pub handle: ash::Instance,
    pub entry: ash::Entry,
    pub debug_utils: ash::extensions::ext::DebugUtils,
    pub utils_messenger: vk::DebugUtilsMessengerEXT
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
}

impl AbstractInstance for VkInstance
{
    fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>
    {
        let handle = window.vk_create_surface(&self.entry, &self.handle, None);
        let loader = ash::extensions::khr::Surface::new(&self.entry, &self.handle);

        Ok(Surface { internal: Rc::new(VkSurface { handle, loader }) })
    }

    fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, ()>
    {
        let mut physical_devices: Vec<PhysicalDevice> = Vec::new();

        for handle in unsafe { self.handle.enumerate_physical_devices() }.unwrap().iter()
        {
            let internal: Rc<dyn AbstractPhysicalDevice> = Rc::new(VkPhysicalDevice { handle: *handle });
            physical_devices.push(PhysicalDevice { internal });
        }

        Ok(physical_devices)
    }

    fn get_physical_device_properties(&self, physical_device: &PhysicalDevice) -> Result<PhysicalDeviceProperties, ()>
    {
        let vk_physical_device = physical_device.downcast_ref::<VkPhysicalDevice>().unwrap();
        let vk_properties = unsafe { self.handle.get_physical_device_properties(vk_physical_device.handle) };

        Ok(PhysicalDeviceProperties
        {
            vendor_id: vk_properties.vendor_id,
            device_id: vk_properties.device_id,
            device_type: match vk_properties.device_type
            {
                vk::PhysicalDeviceType::CPU => DeviceType::CPU,
                vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGPU,
                vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGPU,
                vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGPU,
                _ => DeviceType::Other
            },
            device_name: unsafe { std::ffi::CStr::from_ptr(vk_properties.device_name.as_ptr()) }.to_str().unwrap().to_owned()
        })
    }

    fn create_logical_device(&self, physical_device: &PhysicalDevice) -> Result<Device, ()>
    {
        let vk_physical_device = physical_device.downcast_ref::<VkPhysicalDevice>().unwrap();
        let queue_family_properties = unsafe { self.handle.get_physical_device_queue_family_properties(vk_physical_device.handle) };

        let queue_family_index: u32 =
        {
            let mut found_index: Option<u32> = None;

            for (index, queue_family) in queue_family_properties.iter().enumerate()
            {
                if queue_family.queue_count > 0 &&
                    queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
                    queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
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
        let handle = unsafe { self.handle.create_device(vk_physical_device.handle, &device_create_info, None).unwrap() };

        Ok(Device { internal: Rc::new(VkDevice { handle, queue_family_index }) })
    }
}

#[derive(Clone)]
pub struct VkDevice
{
    pub handle: ash::Device,
    pub queue_family_index: u32
}

impl AbstractDevice for VkDevice
{
    fn as_any(&self) -> &dyn Any { self }

    fn get_device_queue(&self) -> Result<Queue, ()>
    {
        let handle = unsafe { self.handle.get_device_queue(self.queue_family_index, 0) };
        Ok(Queue { internal: Rc::new(VkQueue { handle }) })
    }
}

#[derive(Clone)]
pub struct VkPhysicalDevice
{
    pub handle: ash::vk::PhysicalDevice
}

impl VkPhysicalDevice
{
    
}

impl AbstractPhysicalDevice for VkPhysicalDevice
{
    fn as_any(&self) -> &dyn Any { self }
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

impl VkSwapchain
{
    
}

#[derive(Clone)]
pub struct VkShaderModule
{
    pub handle: vk::ShaderModule
}

impl VkShaderModule
{
    
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
