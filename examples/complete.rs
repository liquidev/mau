use mau::config::WindowConfig;
use serde::{Deserialize, Serialize};

struct App;

impl mau::AppSetup for App {
    type Config = Config;
    type Language = ();
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    language: String,
    window: Option<WindowConfig>,
}

impl mau::AppConfig for Config {
    fn app_name() -> &'static str {
        "mau-example-complete"
    }

    fn language(&self) -> &str {
        &self.language
    }

    fn window_config(&self) -> &Option<mau::config::WindowConfig> {
        &self.window
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "en-US".to_string(),
            window: None,
        }
    }
}

mau::config_module!(Config, config);

struct State;

impl mau::AppState for State {
    fn process(&mut self, args: mau::StateArgs) {
        args.ui.fill(paws::rgb(0, 0, 255));
    }

    fn next_state(self: Box<Self>, renderer: &mut mau::ui::Backend) -> Box<dyn mau::AppState> {
        self
    }
}

fn main() {
    config::load_or_create().unwrap();
    mau::App::<App>::new(&config::config(), State)
        .unwrap()
        .run();
}
