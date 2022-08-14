use thiserror::Error;

#[derive(Debug)]
pub struct ContextConfiguration {
    pub msaa: u16,
    pub error: String,
}

#[derive(Debug)]
pub struct TriedConfigurations(pub Vec<ContextConfiguration>);

impl std::fmt::Display for TriedConfigurations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ContextConfiguration { msaa, error } in &self.0 {
            writeln!(f, "- MSAA: {msaa}, failed with '{error}'")?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create OpenGL context. \nTried the following configurations, none of which seem to be supported:\n{0}\nTry updating your graphics drivers. If that doesn't help, the app is too new to run on your hardware!")]
    CannotInitializeBackend(TriedConfigurations),
    #[error("FreeType error: {0}")]
    CannotInitializeFreetype(#[from] freetype::Error),
    #[error("OpenGL context error: {0}")]
    Context(#[from] glutin::ContextError),
}
