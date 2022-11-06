mod software;
mod vulkan;
mod opengl;

use std::{any::Any, fmt, io::Read, rc::Rc};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
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
pub struct Operations
{
    pub load_op: LoadOp,
    pub store_op: StoreOp
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ShaderModuleSource
{
    Glsl(String),
    Spirv(Vec<u32>)
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ShaderModuleCreateInfo
{
    pub stage: ShaderStage,
    pub source: ShaderModuleSource
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ImageLayout
{
    General,
    ColorAttachmentOptimal,
    DepthStencilAttachmentOptimal,
    StencilStencilReadOnlyOptimal,
    ShaderReadOnlyOptimal,
    TransferSrcOptimal,
    TransferDstOptimal,
    Preinitialized,
    PresentSrc
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Format
{
    Undefined,
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R16Uint,
    R16Sint,
    R16Unorm,
    R16Snorm
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct AttachmentDescription
{
    pub format: u32, // TODO(quigly): change this to enum!
    pub samples: u32,
    pub operations: Operations,
    pub stencil_operations: Operations,
    pub initial_layout: Option<ImageLayout>,
    pub final_layout: ImageLayout
}

mod PipelineStageFlags
{
    const TOP_OF_PIPE:                    u32 = 0b00000001;
    const DRAW_INDIRECT:                  u32 = 0b00000010;
    const VERTEX_INPUT:                   u32 = 0b00000011;
    const VERTEX_SHADER:                  u32 = 0b00000100;
    const TESSELLATION_CONTROL_SHADER:    u32 = 0b00000101;
    const TESSELLATION_EVALUATION_SHADER: u32 = 0b00000110;
    const GEOMETRY_SHADER:                u32 = 0b00000111;
    const FRAGMENT_SHADER:                u32 = 0b00001000;
    const EARLY_FRAGMENT_TESTS:           u32 = 0b00001001;
    const LATE_FRAGMENT_TESTS:            u32 = 0b00001010;
    const COLOR_ATTACHMENT_OUTPUT:        u32 = 0b00001011;
    const COMPUTE_SHADER:                 u32 = 0b00001100;
    const TRANSFER_BIT:                   u32 = 0b00001101;
    const BOTTOM_OF_PIPE:                 u32 = 0b00001110;
    const HOST:                           u32 = 0b00001111;
    const ALL_GRAPHICS:                   u32 = 0b00010000;
    const ALL_COMMANDS:                   u32 = 0b00010001;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SubpassDependency
{
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: u32,
    pub dst_stage_mask: u32,
    pub src_access_mask: u32,
    pub dst_access_mask: u32
}

pub struct RenderPassCreateInfo
{
    //pub attachments: Vec<AttachmentDescription>,
    //pub subpasses: Vec<TODO>
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ShaderModuleError
{
    CompilationFailed,
}

#[derive(Debug, Copy, Clone)]
pub struct SwapchainCreateInfo
{
    pub present_mode: PresentMode
}

pub trait AbstractInstance
{
    fn as_any(&self) -> &dyn Any;
    fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>;
    fn create_device(&self, surface: &Surface) -> Result<Device, ()>;
    fn create_swapchain(&self, device: &Device, surface: &Surface, create_info: &SwapchainCreateInfo) -> Result<Swapchain, ()>;
}

pub trait AbstractDevice
{
    fn as_any(&self) -> &dyn Any;
    fn get_device_queue(&self) -> Result<Queue, ()>;
    fn get_physical_device_properties(&self) -> Result<PhysicalDeviceProperties, ()>;
    fn create_shader_module(&self, create_info: &ShaderModuleCreateInfo) -> Result<ShaderModule, ShaderModuleError>;
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

pub trait AbstractShaderModule
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
    pub fn new(api: API, window: &qpl::Window) -> Result<Self, InstanceError>
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
            /*API::OpenGL =>
            {
                match opengl::GlInstance::new(api, window)
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
            },*/
            _ => { todo!() }
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }

    pub fn create_surface(&self, window: &qpl::Window) -> Result<Surface, ()>
    {
        self.internal.create_surface(window)
    }

    pub fn create_device(&self, surface: &Surface) -> Result<Device, ()>
    {
        self.internal.create_device(surface)
    }

    pub fn create_swapchain(&self, device: &Device, surface: &Surface, create_info: &SwapchainCreateInfo) -> Result<Swapchain, ()>
    {
        self.internal.create_swapchain(device, surface, create_info)
    }
}

pub struct Device
{
    internal: Box<dyn AbstractDevice>
}

impl Device
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }

    pub fn get_device_queue(&self) -> Result<Queue, ()>
    {
        self.internal.get_device_queue()
    }

    pub fn get_physical_device_properties(&self) -> Result<PhysicalDeviceProperties, ()>
    {
        self.internal.get_physical_device_properties()
    }

    pub fn create_shader_module(&self, create_info: &ShaderModuleCreateInfo) -> Result<ShaderModule, ShaderModuleError>
    {
        self.internal.create_shader_module(create_info)
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

impl Image
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}

pub struct ImageView
{
    internal: Rc<dyn AbstractImageView>
}

impl ImageView
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}

pub struct ShaderModule
{
    internal: Rc<dyn AbstractShaderModule>
}

impl ShaderModule
{
    pub fn downcast_ref<T>(&self) -> Option<&T> where T: Any { self.internal.as_any().downcast_ref::<T>() }
}