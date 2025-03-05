pub mod opaque;

pub struct Controllers {
    pub opaque_controller: opaque::OpaqueController,
}

impl Controllers {
    pub fn new() -> Self {
        let opaque_controller = opaque::OpaqueController::new();

        Self { opaque_controller }
    }
}
