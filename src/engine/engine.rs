use crate::application::Application;

pub trait Engine {
    fn start_application(&self, app: &Application);
}
