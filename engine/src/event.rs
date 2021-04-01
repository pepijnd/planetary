use winit::event::{
    DeviceId, ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta,
    TouchPhase,
};

use crate::Size;

#[derive(Debug, PartialEq)]
pub enum WindowEvent {
    Resized(Size),
    Moved((i32, i32)),
    CloseRequested,
    Destroyed,
    Focused(bool),
    KeyboardInput {
        device_id: DeviceId,
        input: KeyboardInput,
        is_synthetic: bool,
    },
    ModifiersChanged(ModifiersState),
    CursorMoved {
        device_id: DeviceId,
        position: (f64, f64),
    },
    CursorEntered {
        device_id: DeviceId,
    },
    CursorLeft {
        device_id: DeviceId,
    },
    MouseWheel {
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
    MouseInput {
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
    },
    ScaleFactorChanged {
        scale_factor: f64,
        size: Size,
    },
    Other,
}

impl From<winit::event::WindowEvent<'_>> for WindowEvent {
    fn from(event: winit::event::WindowEvent) -> Self {
        match event {
            winit::event::WindowEvent::Resized(size) => WindowEvent::Resized(size.into()),
            winit::event::WindowEvent::Moved(loc) => WindowEvent::Moved(loc.into()),
            winit::event::WindowEvent::CloseRequested => WindowEvent::CloseRequested,
            winit::event::WindowEvent::Destroyed => WindowEvent::Destroyed,
            winit::event::WindowEvent::Focused(focused) => WindowEvent::Focused(focused),
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            },
            winit::event::WindowEvent::ModifiersChanged(state) => {
                WindowEvent::ModifiersChanged(state)
            }
            winit::event::WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => WindowEvent::CursorMoved {
                device_id,
                position: position.into(),
            },
            winit::event::WindowEvent::CursorEntered { device_id } => {
                WindowEvent::CursorEntered { device_id }
            }
            winit::event::WindowEvent::CursorLeft { device_id } => {
                WindowEvent::CursorLeft { device_id }
            }
            winit::event::WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            },
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => WindowEvent::MouseInput {
                device_id,
                state,
                button,
            },
            winit::event::WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => WindowEvent::ScaleFactorChanged {
                scale_factor,
                size: (*new_inner_size).into(),
            },
            _ => WindowEvent::Other,
        }
    }
}
