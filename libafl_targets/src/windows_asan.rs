//! Setup asan death callbback

use libafl::{
    events::{EventFirer, EventRestarter},
    executors::{inprocess::windows_asan_handler::asan_death_handler, Executor, HasObservers},
    feedbacks::Feedback,
    state::{HasClientPerfMonitor, HasCorpus, HasExecutions, HasSolutions},
    HasFeedback, HasObjective, HasScheduler,
};

/// Asan death callback type
pub type CB = unsafe extern "C" fn() -> ();

extern "C" {
    fn __sanitizer_set_death_callback(cb: CB);
}

/// Setup `ASan` callback on windows
///
/// This is needed to intercept `ASan` error exit.
///
/// When we use `AddressSanitizer` on Windows, the crash handler is not called when `ASan` detects an error
/// This is because, on linux, `ASan` runtime raises `SIGABRT` so we can rely on the signal handler
/// but on Windows it simply calls `TerminateProcess`.
/// so we need to call the API by `ASan` to register the callback when `ASan` is about to finish the process.
/// See <https://github.com/AFLplusplus/LibAFL/issues/769>.
///
/// # Safety
/// Calls the unsafe `__sanitizer_set_death_callback` symbol, but should be safe to call otherwise.
pub unsafe fn setup_asan_callback<CF, E, EM, OF, Z>(_executor: &E, _event_mgr: &EM, _fuzzer: &Z)
where
    E: Executor<EM, Z> + HasObservers,
    EM: EventFirer<State = E::State> + EventRestarter<State = E::State>,
    CF: Feedback<E::State>,
    OF: Feedback<E::State>,
    E::State: HasSolutions + HasClientPerfMonitor + HasCorpus + HasExecutions,
    Z: HasObjective<Objective = OF, State = E::State>
        + HasFeedback<Feedback = CF, State = E::State>
        + HasScheduler,
{
    __sanitizer_set_death_callback(asan_death_handler::<CF, E, EM, OF, Z>);
}
