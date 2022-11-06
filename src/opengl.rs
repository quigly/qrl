use crate::*;

pub struct GlInstance
{
    handle: qpl::GLContext
}

impl GlInstance
{
    pub fn new(api: API, window: &qpl::Window) -> Result<GlInstance, InstanceError>
    {
        let create_info = qpl::GLContextCreateInfo
        {
            version: (3, 2),
            ..Default::default()
        };

        match window.gl_create_context(&create_info)
        {
            Ok(handle) =>
            {
                Ok(GlInstance { handle })
            },
            Err(gl_error) =>
            {
                Err(InstanceError::ApiNotSupported)
            }
        }
    }
}

impl AbstractInstance for GlInstance
{
    fn as_any(&self) -> &dyn Any { self }

    fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>
    {
        Ok(Surface { internal: Rc::new(GlSurface { }) })
    }

    fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, ()>
    {
        Ok(vec![ PhysicalDevice { internal: Rc::new(GlPhysicalDevice {  }) } ])
    }

    fn get_physical_device_properties(&self, physical_device: &PhysicalDevice) -> Result<PhysicalDeviceProperties, ()>
    {
        let device_name_ptr = unsafe { gl::GetString(gl::RENDERER) };
        let device_name = unsafe { std::ffi::CStr::from_ptr(device_name_ptr as _) }.to_str().unwrap().to_owned();

        Ok(PhysicalDeviceProperties
        {
            vendor_id: 0,
            device_id: 0,
            device_type: DeviceType::Other,
            device_name
        })
    }

    fn create_logical_device(&self, physical_device: &PhysicalDevice, surface: &Surface) -> Result<Device, ()>
    {
        Ok(Device { internal: Box::new(GlDevice {  }) })
    }

    fn create_swapchain(&self, physical_device: &PhysicalDevice, device: &Device, surface: &Surface) -> Result<Swapchain, ()>
    {
        Ok(Swapchain { internal: Rc::new(GlSwapchain {  }) })
    }
}

pub struct GlSurface
{
    
}

impl AbstractSurface for GlSurface
{
    fn as_any(&self) -> &dyn Any { self }
}

pub struct GlPhysicalDevice
{

}

impl AbstractPhysicalDevice for GlPhysicalDevice
{
    fn as_any(&self) -> &dyn Any { self }
}

pub struct GlDevice
{

}

impl AbstractDevice for GlDevice
{
    fn as_any(&self) -> &dyn Any { self }
    
    fn get_device_queue(&self) -> Result<Queue, ()>
    {
        Ok(Queue { internal: Rc::new(GlQueue {  }) })
    }
    
    fn create_shader_module(&self, create_info: &ShaderModuleCreateInfo) -> Result<ShaderModule, ()>
    {
        todo!()
    }
}

pub struct GlQueue
{

}

impl AbstractQueue for GlQueue
{
    fn as_any(&self) -> &dyn Any { self }
}

pub struct GlSwapchain
{

}

impl AbstractSwapchain for GlSwapchain
{
    fn as_any(&self) -> &dyn Any { self }
}