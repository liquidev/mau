use mau::config::WindowConfig;
use serde::{Deserialize, Serialize};

struct App;

impl mau::AppSetup for App {
    type Config = Config;
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

    fn window_config_mut(&mut self) -> &mut Option<mau::config::WindowConfig> {
        &mut self.window
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

struct State;

type AppContext<'a> = mau::AppContext<'a, App>;

impl mau::AppState<App> for State {
    fn process(&mut self, cx: AppContext) {
        cx.ui.fill(paws::rgb(0, 0, 255));
    }

    fn next_state(self: Box<Self>, renderer: &mut mau::ui::Backend) -> Box<dyn mau::AppState<App>> {
        self
    }
}

fn main() {
    mau::run(State)
}
