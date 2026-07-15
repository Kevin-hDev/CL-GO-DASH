use super::pump_gate::PumpGate;
use objc2::{
    define_class, msg_send, rc::Retained, runtime::NSObjectProtocol, sel, AnyThread, DefinedClass,
};
use objc2_app_kit::NSEventTrackingRunLoopMode;
use objc2_foundation::{
    MainThreadMarker, NSNumber, NSObject, NSObjectNSThreadPerformAdditions, NSRunLoop,
    NSRunLoopCommonModes, NSThread, NSTimer,
};
use std::cell::RefCell;
use std::sync::Arc;

define_class! {
    #[unsafe(super(NSObject))]
    #[ivars = Arc<PumpGate>]
    struct PumpTarget;

    impl PumpTarget {
        #[unsafe(method(scheduleWork:))]
        fn schedule_work(&self, delay: &NSNumber) {
            if !self.ivars().begin_dispatch() {
                return;
            }
            if let Ok(delay_ms) = i64::try_from(delay.integerValue()) {
                with_pump(|pump| pump.schedule_work(delay_ms, self));
            }
            self.finish_dispatch();
        }

        #[unsafe(method(timerTimeout:))]
        fn timer_timeout(&self, _timer: &NSTimer) {
            let _ = self.ivars().request();
            if !self.ivars().begin_dispatch() {
                return;
            }
            with_pump(|pump| pump.timer_timeout(self));
            self.finish_dispatch();
        }
    }

    unsafe impl NSObjectProtocol for PumpTarget {}
}

impl PumpTarget {
    fn new(gate: Arc<PumpGate>) -> Retained<Self> {
        let target = Self::alloc().set_ivars(gate);
        unsafe { msg_send![super(target), init] }
    }

    fn finish_dispatch(&self) {
        if self.ivars().complete_and_requeue() {
            queue_on_thread(self, &NSThread::currentThread(), 0);
        }
    }
}

#[derive(Clone)]
pub(super) struct PumpWake {
    gate: Arc<PumpGate>,
    owner_thread: Retained<NSThread>,
    target: Retained<PumpTarget>,
}

// `performSelector:onThread:` is explicitly safe to call from any thread.
unsafe impl Send for PumpWake {}
unsafe impl Sync for PumpWake {}

impl PumpWake {
    pub(super) fn notify(&self, delay_ms: i64) {
        if self.gate.request() {
            queue_on_thread(&self.target, &self.owner_thread, delay_ms);
        }
    }

    fn start(&self) {
        let _ = self.gate.request();
        queue_on_thread(&self.target, &self.owner_thread, 0);
    }
}

struct NativePump {
    timer: Option<Retained<NSTimer>>,
}

impl NativePump {
    fn schedule_work(&mut self, delay_ms: i64, target: &PumpTarget) {
        self.kill_timer();
        if delay_ms <= 0 {
            cef::do_message_loop_work();
        } else {
            self.set_timer(
                delay_ms.min(super::pump_scheduler::fallback_pump_interval_ms() as i64),
                target,
            );
            return;
        }
        self.set_timer(
            super::pump_scheduler::fallback_pump_interval_ms() as i64,
            target,
        );
    }

    fn timer_timeout(&mut self, target: &PumpTarget) {
        self.kill_timer();
        cef::do_message_loop_work();
        self.set_timer(
            super::pump_scheduler::fallback_pump_interval_ms() as i64,
            target,
        );
    }

    fn set_timer(&mut self, delay_ms: i64, target: &PumpTarget) {
        let timer = unsafe {
            NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
                delay_ms.max(1) as f64 / 1_000.0,
                target,
                sel!(timerTimeout:),
                None,
                false,
            )
        };
        let run_loop = NSRunLoop::currentRunLoop();
        unsafe {
            run_loop.addTimer_forMode(&timer, NSRunLoopCommonModes);
            run_loop.addTimer_forMode(&timer, NSEventTrackingRunLoopMode);
        }
        self.timer = Some(timer);
    }

    fn kill_timer(&mut self) {
        if let Some(timer) = self.timer.take() {
            timer.invalidate();
        }
    }
}

thread_local! {
    static NATIVE_PUMP: RefCell<Option<NativePump>> = const { RefCell::new(None) };
}

pub(super) fn start(gate: Arc<PumpGate>) -> Result<PumpWake, ()> {
    MainThreadMarker::new().ok_or(())?;
    NATIVE_PUMP.with(|slot| {
        if slot.borrow().is_some() {
            return Err(());
        }
        let wake = PumpWake {
            owner_thread: NSThread::currentThread(),
            target: PumpTarget::new(gate.clone()),
            gate,
        };
        *slot.borrow_mut() = Some(NativePump { timer: None });
        wake.start();
        Ok(wake)
    })
}

fn with_pump(action: impl FnOnce(&mut NativePump)) {
    NATIVE_PUMP.with(|slot| {
        if let Some(pump) = slot.borrow_mut().as_mut() {
            action(pump);
        }
    });
}

fn queue_on_thread(target: &PumpTarget, thread: &NSThread, delay_ms: i64) {
    let number = NSNumber::numberWithInteger(delay_ms.clamp(0, isize::MAX as i64) as isize);
    unsafe {
        target.performSelector_onThread_withObject_waitUntilDone(
            sel!(scheduleWork:),
            thread,
            Some(&number),
            false,
        );
    }
}

pub(super) fn stop() {
    NATIVE_PUMP.with(|slot| {
        if let Some(mut pump) = slot.borrow_mut().take() {
            pump.kill_timer();
        }
    });
}
