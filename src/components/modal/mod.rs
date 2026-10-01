pub mod component;

#[allow(unused_imports)]
pub use component::{
    BasicModalContent, Modal, ModalBody, ModalClosed, ModalCommand, ModalFooter, ModalHeader,
    ModalOpened, ModalPlugin, ModalStyle,
    spawn_modal, spawn_modal_with_surface,
};
