#[derive(Debug)]
pub struct ShutdownGuard;

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        log::info!("ShutdownGuard::drop()");
    }
}

pub static SHUTDOWN: ShutdownGuard = ShutdownGuard;
