mod software;
mod vulkan;
mod opengl;

use std::{any::Any, fmt, io::Read, rc::Rc};

pub enum API
{
    Software,
    Vulkan,
    OpenGL
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum DeviceType
{
    Other,
    IntegratedGPU,
    DiscreteGPU,
    VirtualGPU,
    CPU
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum CullMode
{
	Front,
	Back
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum FrontFace
{
	Clockwise,
	CounterClockwise
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ShaderStage
{
    Vertex,
    Fragment,
    Geometry
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Primitive
{
	PointList,
	LineList,
	LineStrip,
	TriangleList,
	TriangleStrip
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum LoadOp
{
    Clear(f32, f32, f32, f32),
    Load,
    DontCare
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum StoreOp
{
    Store,
    DontCare
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum PolygonMode
{
    Fill,
    Line,
    Point
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum PresentMode
{
    Immediate,
    Fifo,
    FifoRelaxed,
    Mailbox
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum IndexFormat
{
    Uint16,
    Uint32
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum BlendFactor
{
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
    Src1Color,
    OneMinusSrc1Color,
    Src1Alpha,
    OneMinusSrc1Alpha
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum InstanceError
{
    ApiNotSupported
}

#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties
{
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: DeviceType,
    pub device_name: String
}

pub trait AbstractInstance
{
    fn as_any(&self) -> &dyn Any;
    fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>;
    fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, ()>;
    fn get_physical_device_properties(&self, physical_device: &PhysicalDevice) -> Result<PhysicalDeviceProperties, ()>;
    fn create_logical_device(&self, physical_device: &PhysicalDevice) -> Result<Device, ()>;
    fn create_swapchain(&self, physical_device: &PhysicalDevice, device: &Device, surface: &Surface) -> Result<Swapchain, ()>;
}

pub trait AbstractPhysicalDevice
{
    fn as_any(&self) -> &dyn Any;
}

pub trait AbstractDevice
{
    fn as_any(&self) -> &dyn Any;
    fn get_device_queue(&self) -> Result<Queue, ()>;
}

pub trait AbstractQueue
{
    fn as_any(&self) -> &dyn Any;
}

pub trait AbstractSurface
{
    fn as_any(&self) -> &dyn Any;
}

pub trait AbstractSwapchain
{
    fn as_any(&self) -> &dyn Any;
}

pub trait AbstractImage
{
    fn as_any(&self) -> &dyn Any;
}

pub trait AbstractImageView
{
    fn as_any(&self) -> &dyn Any;
}

pub struct Instance
{
    api: API,
    internal: Box<dyn AbstractInstance>
}

impl Instance
{
    pub fn new(api: API) -> Result<Self, InstanceError>
    {
        match api
        {
            API::Vulkan =>
            {
                match vulkan::VkInstance::new()
                {
                    Ok(instance) =>
                    {
                        Ok(Self
                        {
                            api,
                            internal: Box::new(instance)
                        })
                    },
                    Err(err) =>
                    {
                        Err(err)
                    }
                }
                
            },
            _ => { todo!() }
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }

    pub fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>
    {
        self.internal.create_surface(window)
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, ()>
    {
        self.internal.enumerate_physical_devices()
    }

    pub fn get_physical_device_properties(&self, physical_device: &PhysicalDevice) -> Result<PhysicalDeviceProperties, ()>
    {
        self.internal.get_physical_device_properties(physical_device)
    }

    pub fn select_physical_device(&self) -> Result<PhysicalDevice, ()>
    {
        let devices = self.internal.enumerate_physical_devices().unwrap();
        Ok(devices[0].clone())
    }

    pub fn create_logical_device(&self, physical_device: &PhysicalDevice) -> Result<Device, ()>
    {
        self.internal.create_logical_device(physical_device)
    }

    pub fn create_swapchain(&self, physical_device: &PhysicalDevice, device: &Device, surface: &Surface) -> Result<Swapchain, ()>
    {
        self.internal.create_swapchain(physical_device, device, surface)
    }
}

#[derive(Clone)]
pub struct PhysicalDevice
{
    internal: Rc<dyn AbstractPhysicalDevice>
}

impl PhysicalDevice
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}

#[derive(Clone)]
pub struct Device
{
    internal: Rc<dyn AbstractDevice>
}

impl Device
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }

    pub fn get_device_queue(&self) -> Result<Queue, ()>
    {
        self.internal.get_device_queue()
    }
}

#[derive(Clone)]
pub struct Queue
{
    internal: Rc<dyn AbstractQueue>
}

impl Queue
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}

#[derive(Clone)]
pub struct Surface
{
    internal: Rc<dyn AbstractSurface>
}

impl Surface
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}

#[derive(Clone)]
pub struct Swapchain
{
    internal: Rc<dyn AbstractSwapchain>
}

impl Swapchain
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}

pub struct Image
{
    internal: Rc<dyn AbstractImage>
}

pub struct ImageView
{
    internal: Rc<dyn AbstractImageView>
}