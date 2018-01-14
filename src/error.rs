
use glust::GlError;
use glutin;

error_chain! {
    foreign_links {
        Glust(GlError);
        GlutinCreation(glutin::CreationError);
        GlutinContext(glutin::ContextError);
        Io(::std::io::Error);
    }
}

pub trait OptErr<T> {
    fn ok(self) -> Result<T>;
}

impl<T> OptErr<T> for Option<T> {
    fn ok(self) -> Result<T> {
        self.ok_or("Missing value".into())
    }
}
