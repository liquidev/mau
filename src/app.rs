//! Setup framework for building an app's event loop.

use std::fmt::Write;
use std::marker::PhantomData;

use arrayvec::ArrayString;
use mau_ui::winit::event::{Event, WindowEvent};
use mau_ui::winit::event_loop::ControlFlow;
use mau_ui::winit::window::CursorIcon;
use mau_ui::{Input, UiRenderFrame};
use native_dialog::{MessageDialog, MessageType};
use paws::{vector, Layout};

use crate::clipboard;
use crate::config::{AppConfig, WindowConfig};
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

pub trait AppSetup: 'static {
    type Config: AppConfig + 'static;

    /// The app's pretty name, used for error reporting.
    fn pretty_name() -> &'static str {
        Self::Config::app_name()
    }

    /// The app's issue tracker link, used for error reporting.
    fn issue_tracker() -> Option<&'static str> {
        None
    }
}

pub struct AppContext<'a, T>
where
    T: AppSetup,
{
    pub ui: &'a mut Ui,
    pub input: &'a mut Input,
    pub config: &'a mut T::Config,
}

/// Trait implemented by all app states.
pub trait AppState<T>
where
    T: AppSetup,
{
    /// Processes a single frame.
    ///
    /// In NetCanv, input handling and drawing are done at the same time, which is called
    /// _processing_ in the codebase.
    fn process(&mut self, args: AppContext<T>);

    /// Returns the next state after this one.
    ///
    /// If no state transitions should occur, this should simply return `self`. Otherwise, another
    /// app state may be constructed, boxed, and returned.
    fn next_state(self: Box<Self>, renderer: &mut Backend) -> Box<dyn AppState<T>>;
}

pub fn try_run<T, S>(init_state: S) -> Result<(), Error>
where
    T: AppSetup,
    S: AppState<T> + 'static,
{
    log::debug!("loading config");
    let mut config = T::Config::load_or_create()?;

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

    let mut ui = Ui::new(renderer);
    let mut input = Input::new();

    let mut state: Option<Box<dyn AppState<T>>> = Some(Box::new(init_state));

    // Initialize the clipboard because we now have a window handle.
    match clipboard::init() {
        Ok(_) => (),
        Err(error) => {
            log::error!("failed to initialize clipboard: {:?}", error);
        }
    }

    log::debug!("init done! starting event loop");

    let (mut last_window_position, mut last_window_size) = {
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    // Ignore resize event if window is maximized, and move event if position is lower than 0,
                    // because it isn't what we want, when saving window's size and position to config file.
                    WindowEvent::Resized(new_size) if !ui.window().is_maximized() => {
                        last_window_size = new_size;
                    }
                    WindowEvent::Moved(new_position)
                        if new_position.x >= 0 && new_position.y >= 0 =>
                    {
                        last_window_position = new_position;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {
                        input.process_event(&event);
                    }
                }
            }

            Event::MainEventsCleared => {
                let window_size = ui.window().inner_size();
                if let Err(error) = ui.render_frame(|ui| {
                    ui.root(
                        vector(window_size.width as f32, window_size.height as f32),
                        Layout::Freeform,
                    );
                    // let mut root_view = View::group_sized(ui);
                    // view::layout::full_screen(&mut root_view);

                    input.set_cursor(CursorIcon::Default);
                    state.as_mut().unwrap().process(AppContext {
                        ui,
                        input: &mut input,
                        config: &mut config,
                    });
                    state = Some(state.take().unwrap().next_state(ui.render()));
                }) {
                    log::error!("render error: {}", error)
                }
                input.finish_frame(ui.window());
            }

            Event::LoopDestroyed => {
                let window = ui.window();
                let position = last_window_position;
                let size = last_window_size;
                let maximized = window.is_maximized();
                // TODO: do this
                config.write(|config| {
                    *config.window_config_mut() = Some(WindowConfig {
                        x: position.x,
                        y: position.y,
                        width: size.width,
                        height: size.height,
                        maximized,
                    });
                });
            }

            _ => (),
        }
    })
}

/// Runs the app with the given initial state.
///
/// In addition to running the app, it also sets up a panic hook and handles any fatal errors raised
/// during the app's runtime. If you don't want that, use [`try_run`].
///
/// Do note that this function does not exit.
pub fn run<T, S>(init_state: S)
where
    T: AppSetup,
    S: AppState<T> + 'static,
{
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Pretty panic messages are only enabled in release mode, as they hinder debugging.
        // #[cfg(not(debug_assertions))]
        {
            let mut title = ArrayString::<64>::new();
            let _ = write!(title, "{} - Fatal Error", T::pretty_name());
            let mut message = ArrayString::<4096>::new();
            let _ = write!(message, "Oh no! A fatal error occured.\n{}", panic_info);
            if let Some(issue_tracker) = T::issue_tracker() {
                let _ = write!(
                    message,
                    "\n\nThis is most definitely a bug, so please file an issue at {issue_tracker}",
                );
            }
            let _ = MessageDialog::new()
                .set_title(&title)
                .set_text(&message)
                .set_type(MessageType::Error)
                .show_alert();
        }
        default_panic_hook(panic_info);
    }));

    match try_run(init_state) {
        Ok(()) => (),
        Err(payload) => {
            log::error!("{payload}");
            // let mut message = String::new();
            // let language = language.unwrap_or_else(|| {
            //     Assets::load_language(Some("en-US")).expect("English language must be present")
            // });
            // let _ = write!(
            //     message,
            //     "{}",
            //     Formatted::new(language.clone(), "failure")
            //         .format()
            //         .with("message", payload.translate(&language))
            //         .done(),
            // );
            // log::error!(
            //     "inner_main() returned with an Err:\n{}",
            //     payload.translate(&language)
            // );
            // MessageDialog::new()
            //     .set_title("NetCanv - Error")
            //     .set_text(&message)
            //     .set_type(MessageType::Error)
            //     .show_alert()
            //     .unwrap();
        }
    }
}
