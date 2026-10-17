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

extern crate self as beverly;

pub use beverly_macros::model;

/// Generates a Bevy plugin that routes messages to handlers and composed actions.
///
/// A plain handler receives `&Message`. In a sequence `[a, b]`, actions return
/// `Result`, and each successful output becomes the next action's input. A
/// fan-out `{a, b}` clones its input for independent branches, waits for all
/// branches, and returns `Ok((a_output, b_output))` in declaration order. Groups
/// nest recursively. Register messages separately with `App::add_message`.
///
/// Fan-out uses Bevy's scoped compute pool. Its captured inputs and results
/// must be `Send`; shared borrows must be `Sync`. Multi-branch owned inputs
/// must be `Clone`. Fan-out branches share an error type; sequential stages
/// may convert earlier errors through `From` into the final stage's error type.
/// Composed errors must implement `Display` and are logged. Remaining sequence
/// stages are skipped, but already-started sibling branches finish. The first
/// branch error in declaration order is selected, and later messages continue.
///
/// Composition is synchronous from the caller's perspective. On single-threaded
/// browser targets, synchronous branches do not run simultaneously. Keep long
/// background work and asynchronous request/response lifecycles explicit.
///
/// ```
/// use bevy::prelude::*;
/// use beverly::controller;
///
/// #[derive(Message)]
/// struct Run(u32);
/// fn load(message: &Run) -> Result<u32, &'static str> { Ok(message.0) }
/// fn double(value: u32) -> Result<u32, &'static str> { Ok(value * 2) }
/// fn increment(value: u32) -> Result<u32, &'static str> { Ok(value + 1) }
/// fn save((left, right): (u32, u32)) -> Result<(), &'static str> {
///     assert_eq!((left, right), (6, 4));
///     Ok(())
/// }
/// controller! {
///     ExampleController {
///         Run => [load, { double, increment }, save],
///     }
/// }
/// let mut app = App::new();
/// app.add_message::<Run>().add_plugins(ExampleController);
/// app.world_mut().write_message(Run(3));
/// app.update();
/// ```
///
/// Incompatible adjacent types are rejected at compile time:
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use beverly::controller;
/// #[derive(Message)]
/// struct Run;
/// fn load(_: &Run) -> Result<u32, &'static str> { Ok(1) }
/// fn save(_: String) -> Result<(), &'static str> { Ok(()) }
/// controller! { InvalidController { Run => [load, save] } }
/// ```
///
/// An owned fan-out input must be clonable:
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use beverly::controller;
/// #[derive(Message)]
/// struct Run;
/// struct MoveOnly;
/// fn load(_: &Run) -> Result<MoveOnly, &'static str> { Ok(MoveOnly) }
/// fn consume(_: MoveOnly) -> Result<(), &'static str> { Ok(()) }
/// controller! { InvalidController { Run => [load, { consume, consume }] } }
/// ```
pub use beverly_macros::controller;

pub mod animation;
pub mod app;
pub mod components;
pub mod icons;
mod plugin;
pub mod prelude;
pub mod primitives;
pub mod rendering;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod theme;

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
        assert_eq!(
            app.world()
                .resource::<bevy::ecs::message::Messages<TestCommand::Run>>()
                .len(),
            1
        );
    }
}
#[cfg(test)]
mod controller_macro_tests {
    use crate::controller;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    use bevy::prelude::*;

    event! { ControllerTest::Ping { value: u32 } }

    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static BORROWED_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn branching_ping(message: &ControllerTest::Ping) -> Result<(), &'static str> {
        BORROWED_CALLS.fetch_add(message.value as usize, Ordering::SeqCst);
        Ok(())
    }

    controller! {
        BorrowedController {
            ControllerTest::Ping => { branching_ping, branching_ping },
        }
    }

    #[test]
    fn controller_composition_root_fanout_borrows_clone_messages() {
        BORROWED_CALLS.store(0, Ordering::SeqCst);
        let mut app = App::new();
        app.add_message::<ControllerTest::Ping>()
            .add_plugins(BorrowedController);
        app.world_mut()
            .write_message(ControllerTest::Ping { value: 3 });
        app.update();
        assert_eq!(BORROWED_CALLS.load(Ordering::SeqCst), 6);
    }

    fn handle_ping(message: &ControllerTest::Ping) {
        CALLS.fetch_add(message.value as usize, Ordering::SeqCst);
    }

    controller! {
        ControllerTestPlugin {
            ControllerTest::Ping => handle_ping,
        }
    }

    #[derive(Message)]
    struct ComposeRequest {
        value: u32,
        trace: Arc<Mutex<Vec<String>>>,
    }

    #[derive(Clone)]
    struct Work {
        value: u32,
        trace: Arc<Mutex<Vec<String>>>,
    }

    fn start(message: &ComposeRequest) -> Result<Work, &'static str> {
        message.trace.lock().unwrap().push("start".to_owned());
        Ok(Work {
            value: message.value,
            trace: message.trace.clone(),
        })
    }

    fn double(mut work: Work) -> Result<Work, &'static str> {
        work.value *= 2;
        work.trace
            .lock()
            .unwrap()
            .push(format!("double:{}", work.value));
        Ok(work)
    }

    fn add_three(mut work: Work) -> Result<Work, &'static str> {
        work.value += 3;
        work.trace
            .lock()
            .unwrap()
            .push(format!("add:{}", work.value));
        Ok(work)
    }

    fn reject_zero(work: Work) -> Result<Work, &'static str> {
        if work.value == 0 {
            work.trace.lock().unwrap().push("reject".to_owned());
            Err("zero is rejected")
        } else {
            Ok(work)
        }
    }

    fn reject(work: Work) -> Result<Work, &'static str> {
        work.trace.lock().unwrap().push("reject".to_owned());
        Err("branch failed")
    }

    fn finish(work: Work) -> Result<(), &'static str> {
        work.trace
            .lock()
            .unwrap()
            .push(format!("finish:{}", work.value));
        Ok(())
    }

    fn merge((left, right): (Work, Work)) -> Result<(), &'static str> {
        left.trace
            .lock()
            .unwrap()
            .push(format!("merge:{}", left.value + right.value));
        Ok(())
    }

    fn number(work: Work) -> Result<u32, &'static str> {
        Ok(work.value)
    }

    fn label(work: Work) -> Result<String, &'static str> {
        Ok(work.value.to_string())
    }

    static NESTED_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn merge_nested(
        ((number, label), doubled_label): ((u32, String), String),
    ) -> Result<(), &'static str> {
        assert_eq!(
            (number, label.as_str(), doubled_label.as_str()),
            (5, "5", "10")
        );
        NESTED_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn finish_with_string_error(work: Work) -> Result<(), String> {
        finish(work).map_err(str::to_owned)
    }

    controller! {
        NestedController {
            ComposeRequest => [start, { {number, label}, [double, label] }, merge_nested],
        }
    }

    controller! {
        ConvertedErrorController {
            ComposeRequest => [start, reject_zero, finish_with_string_error],
        }
    }

    controller! {
        SequentialController {
            ComposeRequest => [start, reject_zero, double, finish],
        }
    }

    controller! {
        CombinedController {
            ComposeRequest => [start, { [double, double], add_three }, merge],
        }
    }

    controller! {
        FailedBranchController {
            ComposeRequest => [start, { reject, double }, merge],
        }
    }

    struct OwnedWork(Work);

    fn owned_start(message: &ComposeRequest) -> Result<OwnedWork, &'static str> {
        start(message).map(OwnedWork)
    }

    fn owned_finish(work: OwnedWork) -> Result<(), &'static str> {
        finish(work.0)
    }

    controller! {
        MoveOnlyController {
            ComposeRequest => [owned_start, owned_finish],
        }
    }

    #[derive(Message)]
    struct ParallelRequest {
        rendezvous: Arc<(Mutex<usize>, Condvar)>,
        completed: Arc<AtomicUsize>,
    }

    fn overlapping_branch(message: &ParallelRequest) -> Result<(), &'static str> {
        let (arrived, ready) = &*message.rendezvous;
        let mut count = arrived.lock().unwrap();
        *count += 1;
        ready.notify_all();
        let (count, _) = ready
            .wait_timeout_while(count, Duration::from_secs(3), |count| *count < 2)
            .unwrap();
        if *count < 2 {
            return Err("parallel branches did not overlap");
        }
        message.completed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    controller! {
        ParallelController {
            ParallelRequest => { overlapping_branch, overlapping_branch },
        }
    }

    #[test]
    fn controller_composition_sequences_values_and_continues_after_errors() {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        app.add_message::<ComposeRequest>()
            .add_plugins(SequentialController);
        for value in [0, 5] {
            app.world_mut().write_message(ComposeRequest {
                value,
                trace: trace.clone(),
            });
        }
        app.update();
        assert_eq!(
            *trace.lock().unwrap(),
            ["start", "reject", "start", "double:10", "finish:10"]
        );
    }

    #[test]
    fn controller_composition_moves_non_clone_intermediates() {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        app.add_message::<ComposeRequest>()
            .add_plugins(MoveOnlyController);
        app.world_mut().write_message(ComposeRequest {
            value: 7,
            trace: trace.clone(),
        });
        app.update();
        assert_eq!(*trace.lock().unwrap(), ["start", "finish:7"]);
    }

    #[test]
    fn controller_composition_joins_nested_outputs_in_declaration_order() {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        app.add_message::<ComposeRequest>()
            .add_plugins(CombinedController);
        app.world_mut().write_message(ComposeRequest {
            value: 5,
            trace: trace.clone(),
        });
        app.update();
        let trace = trace.lock().unwrap();
        assert_eq!(trace.first().unwrap(), "start");
        assert_eq!(trace.last().unwrap(), "merge:28");
        assert!(
            trace.iter().position(|value| value == "double:10").unwrap()
                < trace.iter().position(|value| value == "double:20").unwrap()
        );
        assert!(trace.contains(&"add:8".to_owned()));
    }

    #[test]
    fn controller_composition_waits_for_siblings_but_skips_join_after_branch_error() {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        app.add_message::<ComposeRequest>()
            .add_plugins(FailedBranchController);
        app.world_mut().write_message(ComposeRequest {
            value: 5,
            trace: trace.clone(),
        });
        app.update();
        let trace = trace.lock().unwrap();
        assert_eq!(trace.len(), 3);
        assert!(trace.contains(&"reject".to_owned()));
        assert!(trace.contains(&"double:10".to_owned()));
    }

    #[test]
    fn controller_composition_parallel_branches_overlap() {
        let completed = Arc::new(AtomicUsize::new(0));
        let mut app = App::new();
        app.add_message::<ParallelRequest>()
            .add_plugins(ParallelController);
        app.world_mut().write_message(ParallelRequest {
            rendezvous: Arc::new((Mutex::new(0), Condvar::new())),
            completed: completed.clone(),
        });
        app.update();
        assert_eq!(completed.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn controller_composition_supports_heterogeneous_nested_tuples() {
        NESTED_CALLS.store(0, Ordering::SeqCst);
        let mut app = App::new();
        app.add_message::<ComposeRequest>()
            .add_plugins(NestedController);
        app.world_mut().write_message(ComposeRequest {
            value: 5,
            trace: Arc::new(Mutex::new(Vec::new())),
        });
        app.update();
        assert_eq!(NESTED_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn controller_composition_converts_sequential_errors_into_final_error_type() {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let mut app = App::new();
        app.add_message::<ComposeRequest>()
            .add_plugins(ConvertedErrorController);
        for value in [0, 5] {
            app.world_mut().write_message(ComposeRequest {
                value,
                trace: trace.clone(),
            });
        }
        app.update();
        assert_eq!(
            *trace.lock().unwrap(),
            ["start", "reject", "start", "finish:5"]
        );
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
