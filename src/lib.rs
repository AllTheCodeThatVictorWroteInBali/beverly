//! Beverly is a Rust-native UI system for building high-performance,
//! accessible interfaces with [Bevy](https://bevyengine.org/).
//!
//! ```no_run
//! use bevy::prelude::*;
//! use beverly::prelude::*;
//!
//! fn main() {
//!     App::new().ui(my_ui()).run();
//! }
//!
//! fn my_ui() -> Ui {
//!     ui()
//!         .theme(light_theme())
//!         .center()
//!         .children([text("Hello, World!")])
//! }
//! ```

pub mod animation;
pub mod app;
pub mod components;
pub mod icons;
mod plugin;
pub mod prelude;
pub mod primitives;
pub mod rendering;
pub mod theme;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use plugin::BeverlyPlugin;

/// Declares application messages in a namespaced, strongly typed form.
///
/// The macro generates one Bevy [`Message`] struct for each declaration. The
/// messages still need to be registered with `App::add_message` by the
/// application or plugin that owns them.
///
/// ```
/// use beverly::event;
/// use bevy::prelude::*;
///
/// event! {
///     User::Create {
///         name: String
///     }
/// }
///
/// let mut app = App::new();
/// app.add_message::<User::Create>();
/// ```
#[macro_export]
macro_rules! event {
	(@decl $namespace:ident :: $name:ident { $( $field:ident : $ty:ty ),* $(,)? }) => {
		#[allow(non_snake_case)]
		pub mod $namespace {
			#[derive(::bevy::prelude::Message, Clone, Debug)]
			pub struct $name {
				$(pub $field: $ty),*
			}
		}
	};
	(@decl $namespace:ident :: $name:ident) => {
		#[allow(non_snake_case)]
		pub mod $namespace {
			#[derive(::bevy::prelude::Message, Clone, Debug)]
			pub struct $name;
		}
	};
	($( $namespace:ident :: $name:ident $( { $( $field:ident : $ty:ty ),* $(,)? } )? )*) => {
		$(
			$crate::event!(@decl $namespace :: $name $( { $( $field : $ty ),* } )?);
		)*
	};
}
/// Generates a Bevy plugin that routes messages to application handlers.
///
/// Each handler has the signature `fn(&MessageType)`. Register the message
/// types separately with `App::add_message`, then add the generated controller
/// plugin with `App::add_plugins`.
#[macro_export]
macro_rules! controller {
	($controller:ident { $( $message:path => $handler:path ),* $(,)? }) => {
		pub struct $controller;

		impl ::bevy::prelude::Plugin for $controller {
			fn build(&self, app: &mut ::bevy::prelude::App) {
				app.add_systems(
					::bevy::prelude::Update,
					(
						$(
							|mut messages: ::bevy::prelude::MessageReader<$message>| {
								for message in messages.read() {
									$handler(message);
								}
							}
						),*
					),
				);
			}
		}
	};
}

#[cfg(test)]
mod event_macro_tests {
	#![allow(dead_code)]
	event! {
		TestCommand::Run {
			value: u32,
		}
		TestFact::Finished
	}

	#[test]
	fn event_macro_generates_message_types() {
		let mut app = bevy::app::App::new();
		app.add_message::<TestCommand::Run>();
		app.add_message::<TestFact::Finished>();
		app.world_mut().write_message(TestCommand::Run { value: 7 });
		app.world_mut().write_message(TestFact::Finished);
		assert_eq!(app.world().resource::<bevy::ecs::message::Messages<TestCommand::Run>>().len(), 1);
	}
}
#[cfg(test)]
mod controller_macro_tests {
	use std::sync::atomic::{AtomicUsize, Ordering};

	use bevy::prelude::*;

	event! { ControllerTest::Ping { value: u32 } }

	static CALLS: AtomicUsize = AtomicUsize::new(0);

	fn handle_ping(message: &ControllerTest::Ping) {
		CALLS.fetch_add(message.value as usize, Ordering::SeqCst);
	}

	controller! {
		ControllerTestPlugin {
			ControllerTest::Ping => handle_ping,
		}
	}

	#[test]
	fn controller_macro_routes_messages_to_handlers() {
		CALLS.store(0, Ordering::SeqCst);
		let mut app = App::new();
		app.add_message::<ControllerTest::Ping>()
			.add_plugins(ControllerTestPlugin);
		app.world_mut()
			.write_message(ControllerTest::Ping { value: 3 });
		app.update();
		assert_eq!(CALLS.load(Ordering::SeqCst), 3);
	}
}

