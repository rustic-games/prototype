//! A simple game loop implementation.

#![warn(
    clippy::all,
    clippy::cargo,
    clippy::clone_on_ref_ptr,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::nursery,
    clippy::option_unwrap_used,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::result_unwrap_used,
    clippy::unimplemented,
    clippy::wildcard_enum_match_arm,
    clippy::wrong_pub_self_convention,
    clippy::dbg_macro,
    clippy::use_debug,
    deprecated_in_future,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    warnings
)]

use std::fmt::Debug;
use std::time::{Duration, Instant};

/// Convenience constant, to make the rest of the code a bit easier to parse.
const NANOSECONDS_PER_SECOND: u32 = 1_000_000_000;

/// The _internal_ state of the [`GameLoop`].
///
/// Whenever [`tick()`] is called, the [`State`] goes from [`Idle`], to
/// [`Updating`], [`Rendering`], and finally back to `Idle`.
///
/// This is an internal representation, because the state can never be anything
/// other than `Idle` before and after running `tick()`.
///
#[derive(Debug, PartialEq, Eq)]
enum State {
    /// The `Idle` state represents the state the [`GameLoop`] is in right
    /// before calling [`tick()`], and after that method is completed.
    ///
    /// Meaning, when inspecting the state of the GameLoop outside of that
    /// method, it should never be anything other than `Idle`.
    Idle,

    /// The `Updating` state represents the state of the [`GameLoop`] when it is
    /// being updated by calling [`Updater#update()`].
    Updating,

    /// The `Rendering` state represents the state of the [`GameLoop`] when it
    /// is being updated by calling [`Renderer#render()`].
    Rendering,
}

/// The trait responsible for _updating_ the state of the game world.
///
/// It requires a single method [`update()`] to be implemented.
///
/// This trait goes hand-in-hand with the trait to render the state of the game
/// world: [`Renderer`].
///
pub trait Updater: Debug {
    /// The error type returned when updating fails.
    type Error: std::error::Error;

    /// What this method does is up to the implementer, but by convention, it
    /// should focus on updating the _state_ of the game world, not the _visual
    /// representation_.
    ///
    /// If this method returns an error, the game loop will bubble up that error
    /// to the callee of [`GameLoop::tick`].
    fn update(&mut self) -> Result<(), Self::Error>;
}

/// The trait responsible for _rendering_ the state of the game world.
///
/// It requires a single method [`render()`] to be implemented.
///
/// This trait goes hand-in-hand with the trait to update the state of the game
/// world: [`Updater`].
///
pub trait Renderer: Debug {
    /// The error type returned when rendering fails.
    type Error: std::error::Error;

    /// What this method does is up to the implementer, but by convention, it
    /// should focus on updating the _visual representation_ of the game world,
    /// not the _state_.
    ///
    /// The `remainder` represents the position (0.0 >= remainder < 1.0) between
    /// the last game state update and the next update. This value can be used
    /// to interpolate the current game state and render the state accordingly.
    ///
    /// If this method returns an error, the game loop will bubble up that error
    /// to the callee of [`GameLoop::tick`].
    fn render(&mut self, remainder: f32) -> Result<(), Self::Error>;
}

/// The main game loop.
///
/// It takes ownership of the game state, and calls its `update` and `render`
/// methods when needed.
///
/// The loop does not advance by itself, you are still required to call `tick`
/// to execute the next game tick.
///
/// You can choose to update the game as fast as possible by calling `tick` in
/// an infinite loop, limit the max frames by sleeping between ticks, or
/// manually advance the game state by calling `tick` whenever you need to, for
/// example when running tests.
#[derive(Debug)]
pub struct GameLoop<T>
where
    T: Updater + Renderer + Debug,
{
    /// The state of the game.
    state: T,

    /// The minimum amount of time that needs to pass before we trigger a game
    /// state update. This is a fixed delta, to give us a predictable game
    /// simulation, and decouple our simulation from the capabilities of the
    /// host in terms of rendering performance.
    ///
    /// Think of it like this: after every frame render, we've given ourselves
    /// some time to perform game state updates. We'll perform those updates at
    /// the interval defined here, and we'll continue those updates for as long
    /// as we don't have to render the next frame.
    update_interval: Duration,

    /// Data associated with the previous tick run.
    ///
    /// Based on this data, the game loop determines how many updates need to
    /// happen before the next render is triggered.
    previous_tick: Option<Tick>,

    /// `accumulated_time` is the total time available for the update handler to
    /// run. After each update step, we subtract the `update_interval` from the
    /// remaining `accumulated_time`.
    ///
    /// When the accumulated time falls below the update interval value, it
    /// means there is no more room for another game update, so we render
    /// another frame to the screen, informing the renderer how much
    /// (normalised) residue accumulated time is left, to allow the renderer to
    /// interpolate the expected game state between the last and next update
    /// call.
    ///
    /// Say for example that we moved to position X = 10 on the last update, and
    /// the following is true:
    ///
    /// * we move 2X per update
    ///
    /// * we update the game state 100 times per second (so we need 10
    ///   milliseconds per update)
    ///
    /// * our `accumulated_time` has 5 milliseconds remaining (remember, we
    ///   _need_ 10 milliseconds to update the game state, so the last 5
    ///   milliseconds are kept around)
    ///
    /// We now know that if we had 10 milliseconds remaining, the character
    /// would've moved to X = 12. But since we only had 5 milliseconds left, the
    /// character position wasn't updated in the last cycle. However, as soon as
    /// we add 5 more milliseconds to our accumulator in the next cycle, it will
    /// move to that X = 12 position.
    ///
    /// Given this, the normalized accumulated value will be `0.5`, since we
    /// have half of the time needed to perform another game state update.
    ///
    /// So, instead of rendering our character as "stopped" on X = 10, we'll
    /// instead interpolate that we were at X = 10 in the last update, the
    /// character moves at 2X per update, so we multiply that value by `0.5` to
    /// guess that even though the game state doesn't reflect this yet, the
    /// character is currently at X = 11, and render it in that position
    /// accordingly.
    ///
    /// If, during the next update cycle, the character is moved to X = 12, we
    /// can render the character there, and we've had three frames, the first
    /// one rendering the character at position 10, the second frame at 11,
    /// and the third at 12.
    ///
    /// If, however, it turns out the player instead instructed the character to
    /// stop after the first frame (when the game still had the character
    /// positioned at X = 10), we'll have to move the character back on the
    /// screen. This will cause a (mostly unnoticeable) "stutter", but the fact
    /// is that most of the time, the character would have ended up at X = 12,
    /// making it a worthy trade off to have a once-every-while bad
    /// interpolation, instead of constantly stuttering images due to not
    /// interpolating the remaining accumulated update time every cycle.
    ///
    /// TODO: this should probably be converted to raw numbers at some point,
    /// for performance reasons, but not until we measure the results. For now
    /// this is fine.
    accumulated_time: Duration,
}

/// Represents a single "tick" of the game loop.
#[derive(Debug)]
struct Tick {
    /// Whenever a new "tick" is started, this field is set to the current
    /// timestamp. An [`Instant`] is used to record the time, so it can only be
    /// used to measure the duration between two ticks, not to record _when_ a
    /// tick was started.
    started_at: Instant,

    /// The state that the tick is currently in.
    state: State,
}

/// The error state of the game loop.
///
/// If either the `Updater::update` or `Renderer::render` method returns an
/// error when calling `tick`, it is wrapped into this game loop error type, and
/// returned.
#[derive(Debug)]
pub enum Error<T>
where
    T: Updater + Renderer,
{
    /// The update call produced an error.
    Update(<T as Updater>::Error),

    /// The render call produced an error.
    Render(<T as Renderer>::Error),
}

impl Default for Tick {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            state: State::Idle,
        }
    }
}

impl<T> GameLoop<T>
where
    T: Updater + Renderer,
{
    /// Create a new game loop with the given state.
    pub fn new(state: T) -> Self {
        // Sets the game state update to a fixed interval. This is what
        // decouples your game update behaviour from the speed at which the game
        // is rendered to the screen (FPS).
        //
        // # See Also
        //
        // * https://www.koonsolo.com/news/dewitters-gameloop/
        // * https://gafferongames.com/post/fix_your_timestep/
        // * http://gameprogrammingpatterns.com/game-loop.html
        //
        // TODO: move this into a configuration struct, or add a builder.
        let updates_per_second = 100;

        Self {
            state,
            previous_tick: None,
            accumulated_time: Duration::default(),
            update_interval: Duration::from_nanos(
                u64::from(NANOSECONDS_PER_SECOND) / updates_per_second,
            ),
        }
    }

    /// A tick is a single "step" forward for the entire state of the game.
    ///
    /// Depending on the game state, calling this method will call the
    /// `Updater#update` method zero, one or multiple times, and will always
    /// call the `Renderer#render` method exactly once.
    pub fn tick(&mut self) -> Result<(), Error<T>> {
        use State::*;

        // Create a new tick instance, to keep track of this tick's progress.
        let mut tick = Tick::default();
        debug_assert_eq!(tick.state, Idle);

        // We'll continue to drive the game state forward, until we've completed
        // all the work for this tick.
        loop {
            match tick.state {
                // The tick is about to start running, so we check how long ago
                // the last tick ran, to determine the speed of the game loop,
                // and set the amount of times the updater should run to catch
                // up.
                Idle => {
                    if let Some(tick) = &self.previous_tick {
                        let previous_tick_duration = tick.started_at.elapsed();
                        self.accumulated_time += previous_tick_duration;
                    }

                    tick.state = Updating;
                }

                // If enough time has accumulated since the last tick, run the
                // updater, until it has drained the accumulated time.
                //
                // The required accumulated time depends on the configured
                // updates per second. If set to 100, we have a budget of 10
                // milliseconds per update, so `accumulated_time` needs to be 10
                // milliseconds or more to perform another update.
                //
                // After updating the game, we keep the [`GameState`] set to
                // `Updating`, and we try to update the game again, until we run
                // out of `accumuated_time`.
                Updating if self.accumulated_time >= self.update_interval => {
                    self.state.update().map_err(Error::Update)?;
                    self.accumulated_time -= self.update_interval;
                }

                // Once we run out of time to update the game state, move on to
                // rendering.
                Updating => {
                    tick.state = Rendering;
                }

                // Call the renderer.
                //
                // While the `accumulated_time` budget wasn't large enough to
                // perform another game update, chances are it wasn't exactly
                // zero once we were done updating the game. This means we're
                // about to render the game in-between two game updates.
                //
                // We pass the "remainder" (a value between 0.0 and 1.0) between
                // the last update, and the expected next update to the
                // [`Renderer`], to allow for visual interpolation of the game
                // state.
                Rendering => {
                    self.state.render(self.remainder()).map_err(Error::Render)?;
                    self.previous_tick = Some(tick);

                    // We're done with this tick, exit the method.
                    return Ok(());
                }
            }
        }
    }

    /// A helper method to get the remainder stored in the game loop.
    ///
    /// This is meant to aid in unit testing the state of the game by inspecting
    /// how much time is still stored as the remainder of the game loop.
    pub fn remainder(&self) -> f32 {
        let remainder = as_secs_f32(self.accumulated_time) / as_secs_f32(self.update_interval);
        debug_assert!((remainder >= 0.0) && (remainder < 1.0));

        remainder
    }

    /// A helper method to inspect the game state.
    ///
    /// This is meant to aid in unit testing the state of the game by allowing
    /// inspection (or mutation) of the game state after performing a game tick.
    pub fn state(&mut self) -> &mut T {
        &mut self.state
    }

    /// A helper method to increase the accumulated time by a fixed amount.
    ///
    /// This is meant to aid in unit testing the state of the game by forcing
    /// the updater to run a fixed amount of times when triggering another game
    /// tick.
    pub fn add_accumulated_time(&mut self, add: Duration) {
        self.accumulated_time += add;
    }
}

/// Convert a duration to fractional seconds.
///
/// See: <https://github.com/rust-lang/rust/pull/62756>
#[allow(clippy::cast_precision_loss)]
fn as_secs_f32(duration: Duration) -> f32 {
    (duration.as_secs() as f32) + (duration.subsec_nanos() as f32) / (NANOSECONDS_PER_SECOND as f32)
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct State {
        update: usize,
        render: usize,
    }

    impl Updater for State {
        type Error = std::io::Error;

        fn update(&mut self) -> Result<(), Self::Error> {
            self.update += 1;
            Ok(())
        }
    }

    impl Renderer for State {
        type Error = std::io::Error;

        fn render(&mut self, _remainder: f32) -> Result<(), Self::Error> {
            self.render += 1;
            Ok(())
        }
    }

    #[test]
    fn test_game_loop_state() {
        let mut game_loop = GameLoop::new(State {
            update: 1,
            render: 2,
        });

        assert_eq!(game_loop.state().update, 1);
        assert_eq!(game_loop.state().render, 2);
    }

    #[test]
    fn test_game_loop_tick_drains_accumulated_time() {
        let mut game_loop = GameLoop::new(State::default());

        // we run at 100 FPS, so update the game state every 10ms
        game_loop.add_accumulated_time(Duration::from_millis(10));
        game_loop.tick().unwrap();
        assert_eq!(game_loop.state().update, 1);

        // At the last tick, the updater ran once, and drained all accumulated
        // time. We add 6 more milliseconds, bringing the total to 6, so no new
        // update is triggered.
        game_loop.add_accumulated_time(Duration::from_millis(6));
        game_loop.tick().unwrap();
        assert_eq!(game_loop.state().update, 1);

        // We still have 6 milliseconds accumulated, by adding 16 more, we end
        // up with 22, so the updater runs twice, leaving 2 accumulated
        // milliseconds.
        game_loop.add_accumulated_time(Duration::from_millis(16));
        game_loop.tick().unwrap();
        assert_eq!(game_loop.state().update, 3);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_game_loop_remainder() {
        let mut game_loop = GameLoop::new(State::default());

        game_loop.add_accumulated_time(Duration::from_millis(9));
        assert_eq!(game_loop.remainder(), 0.9);
    }

    #[test]
    #[should_panic]
    fn test_game_loop_invalid_remainder() {
        let mut game_loop = GameLoop::new(State::default());

        game_loop.add_accumulated_time(Duration::from_millis(10));

        // The remainder has to be 0.0 or higher, and lower than 1.0 to be
        // valid. The only way this invalid state can be triggered is if the
        // `add_accumulated_time` is used to manually add 10 or more
        // milliseconds, without using `tick` to consume that accumulated time
        // down to below 10.
        let _ = game_loop.remainder();
    }

    #[test]
    fn test_game_loop_tick_runs_renderer() {
        let mut game_loop = GameLoop::new(State::default());

        game_loop.tick().unwrap();

        assert_eq!(game_loop.state().render, 1);
    }
}
