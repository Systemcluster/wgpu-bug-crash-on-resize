use futures::executor::block_on;
use tracing::metadata::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;
use wgpu::{
    BackendBit, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode,
    Queue, RequestAdapterOptions, Surface, SwapChain, SwapChainDescriptor, SwapChainError,
    SwapChainFrame, TextureFormat, TextureUsage,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

pub fn create_window(eventloop: &winit::event_loop::EventLoop<()>) -> Window {
    let builder = WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1024, 1024))
        .with_min_inner_size(LogicalSize::new(1024, 1024))
        .with_title(env!("CARGO_PKG_NAME"))
        .with_transparent(false)
        .with_decorations(true);
    builder.build(&eventloop).unwrap()
}

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub swapchain: Option<SwapChain>,
    pub frame: Option<SwapChainFrame>,
}

impl Renderer {
    pub fn new(device: Device, queue: Queue, surface: Surface) -> Self {
        Self {
            device,
            queue,
            surface,
            swapchain: None,
            frame: None,
        }
    }

    pub fn swap(&mut self) -> Result<(), SwapChainError> {
        if self.swapchain.is_some() {
            self.frame = Some(self.swapchain.as_ref().unwrap().get_current_frame()?);
        }
        Ok(())
    }

    pub fn present(&mut self) {
        self.frame = None;
    }

    pub fn create_swap_chain(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            self.swapchain = None;
            return;
        }
        self.swapchain = Some(self.device.create_swap_chain(
            &self.surface,
            &SwapChainDescriptor {
                usage: TextureUsage::RENDER_ATTACHMENT,
                format: TextureFormat::Bgra8Unorm,
                width,
                height,
                present_mode: PresentMode::Mailbox,
            },
        ));
    }
}

fn main() {
    LogTracer::init().unwrap();
    let collector = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default().add_directive(LevelFilter::WARN.into()))
        .compact()
        .finish();
    tracing::subscriber::set_global_default(collector).unwrap();

    let eventloop = EventLoop::new();
    let window = create_window(&eventloop);

    let backend = BackendBit::DX11;
    let instance = Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter_options = RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    };
    let adapter = block_on(instance.request_adapter(&adapter_options)).unwrap();
    let device_limits = Limits {
        max_push_constant_size: 128,
        ..Limits::default()
    };
    let device_features = Features::default() | Features::PUSH_CONSTANTS;
    let device_descriptor = DeviceDescriptor {
        limits: device_limits,
        features: device_features,
        shader_validation: true,
        label: None,
    };
    let (device, queue) = block_on(
        adapter.request_device(&device_descriptor, Some(&std::path::Path::new("./trace"))),
    )
    .unwrap();

    window.set_visible(true);

    let mut renderer = Renderer::new(device, queue, surface);

    eventloop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                if *control_flow == ControlFlow::Exit {
                    return;
                }
                renderer.swap().unwrap();
                renderer.present();
            }
            Event::RedrawRequested(_) => {}
            Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                if window_id == window.id() {
                    match window_event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(new_inner_size) => {
                            renderer.create_swap_chain(new_inner_size.width, new_inner_size.height);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // renderer.create_swap_chain(new_inner_size.width, new_inner_size.height);
                        }
                        _ => (),
                    }
                }
            }
            _ => {}
        }
    });
}
