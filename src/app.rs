//! Setup framework for building an app's event loop.

use std::marker::PhantomData;

use mau_ui::winit::event::{Event, WindowEvent};
use mau_ui::winit::event_loop::ControlFlow;
use mau_ui::winit::window::CursorIcon;
use mau_ui::{Input, UiRenderFrame};
use paws::{vector, Layout};

use crate::clipboard;
use crate::config::AppConfig;
use crate::error::Error;
use mau_ui::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use mau_ui::Backend;

#[cfg(target_os = "linux")]
use mau_ui::winit::platform::unix::WindowBuilderExtUnix;

/// The paws UI layout framework, specialized for the selected backend.
pub type Ui = paws::Ui<mau_ui::Backend>;

pub trait AppSetup {
    type Config: AppConfig;
    type Language;
}

/// An application with a window, UI framework, event loop, everything basically.
pub struct App<T>
where
    T: AppSetup,
{
    pub ui: Ui,
    pub input: Input,
    pub event_loop: EventLoop<()>,

    state: Option<Box<dyn AppState>>,

    last_window_position: PhysicalPosition<i32>,
    last_window_size: PhysicalSize<u32>,

    _todo: PhantomData<T>,
    // pub language: T::Language,
}

impl<T> App<T>
where
    T: AppSetup,
{
    pub fn new<S>(config: &T::Config, init_state: S) -> Result<Self, Error>
    where
        S: AppState + 'static,
    {
        // Set up the winit event loop and open the window.
        log::debug!("opening window");
        let event_loop = EventLoop::new();
        let window_builder = {
            let b = WindowBuilder::new()
                .with_inner_size(PhysicalSize::<u32>::new(1024, 600))
                .with_title("NetCanv")
                .with_resizable(true);
            let b = if let Some(window) = config.window_config() {
                b.with_inner_size(PhysicalSize::new(window.width, window.height))
            } else {
                b
            };
            // On Linux, winit doesn't seem to set the app ID properly so Wayland compositors can't tell
            // our window apart from others.
            #[cfg(target_os = "linux")]
            let b = b.with_app_id(T::Config::app_name().to_string());

            b
        };

        // Build the render backend.
        log::debug!("initializing render backend");
        let renderer = Backend::new(window_builder, &event_loop).map_err(Error::Backend)?;
        // Position and maximize the window.
        // NOTE: winit is a bit buggy and WindowBuilder::with_maximized does not
        // make window maximized, but Window::set_maximized does.
        if let Some(window) = config.window_config() {
            renderer
                .window()
                .set_outer_position(PhysicalPosition::new(window.x, window.y));
            renderer.window().set_maximized(window.maximized);
        }

        let ui = Ui::new(renderer);

        // Initialize the clipboard because we now have a window handle.
        match clipboard::init() {
            Ok(_) => (),
            Err(error) => {
                log::error!("failed to initialize clipboard: {:?}", error);
            }
        }

        log::debug!("init done! starting event loop");

        let (last_window_position, last_window_size) = {
            if let Some(window) = &config.window_config() {
                let size = PhysicalSize::new(window.width, window.height);
                let position = PhysicalPosition::new(window.x, window.y);
                (position, size)
            } else {
                let size = ui.window().inner_size();
                let position = ui.window().outer_position().unwrap_or_default();
                (position, size)
            }
        };

        Ok(Self {
            ui,
            event_loop,
            input: Input::new(),

            state: Some(Box::new(init_state)),

            last_window_position,
            last_window_size,

            _todo: PhantomData,
        })
    }

    /// Enters the event loop and starts running the application.
    pub fn run(mut self) -> ! {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        // Ignore resize event if window is maximized, and move event if position is lower than 0,
                        // because it isn't what we want, when saving window's size and position to config file.
                        WindowEvent::Resized(new_size) if !self.ui.window().is_maximized() => {
                            self.last_window_size = new_size;
                        }
                        WindowEvent::Moved(new_position)
                            if new_position.x >= 0 && new_position.y >= 0 =>
                        {
                            self.last_window_position = new_position;
                        }
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {
                            self.input.process_event(&event);
                        }
                    }
                }

                Event::MainEventsCleared => {
                    let window_size = self.ui.window().inner_size();
                    if let Err(error) = self.ui.render_frame(|ui| {
                        ui.root(
                            vector(window_size.width as f32, window_size.height as f32),
                            Layout::Freeform,
                        );
                        // let mut root_view = View::group_sized(ui);
                        // view::layout::full_screen(&mut root_view);

                        self.input.set_cursor(CursorIcon::Default);
                        self.state.as_mut().unwrap().process(StateArgs {
                            ui,
                            input: &mut self.input,
                        });
                        self.state = Some(self.state.take().unwrap().next_state(ui.render()));
                    }) {
                        log::error!("render error: {}", error)
                    }
                    self.input.finish_frame(self.ui.window());
                }

                Event::LoopDestroyed => {
                    let window = self.ui.window();
                    let position = self.last_window_position;
                    let size = self.last_window_size;
                    let maximized = window.is_maximized();
                    // TODO: do this
                    // config::write(|config| {
                    //     config.window = Some(WindowConfig {
                    //         x: position.x,
                    //         y: position.y,
                    //         width: size.width,
                    //         height: size.height,
                    //         maximized,
                    //     });
                    // });
                }

                _ => (),
            }
        });
    }
}

/// Arguments passed to app states.
#[non_exhaustive]
pub struct StateArgs<'a, 'b> {
    pub ui: &'a mut Ui,
    pub input: &'b mut Input,
}

/// Trait implemented by all app states.
pub trait AppState {
    /// Processes a single frame.
    ///
    /// In NetCanv, input handling and drawing are done at the same time, which is called
    /// _processing_ in the codebase.
    fn process(&mut self, args: StateArgs);

    /// Returns the next state after this one.
    ///
    /// If no state transitions should occur, this should simply return `self`. Otherwise, another
    /// app state may be constructed, boxed, and returned.
    fn next_state(self: Box<Self>, renderer: &mut Backend) -> Box<dyn AppState>;
}
