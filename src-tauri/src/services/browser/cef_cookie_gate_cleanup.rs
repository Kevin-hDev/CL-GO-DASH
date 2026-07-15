use super::cef_cookie_gate::{CookieGateContext, PROBE_COOKIE_NAME, PROBE_COOKIE_URL};
use super::cookie_store_probe::cookie_store_hides_probe;
use cef::*;

pub(super) fn flush_and_check(context: CookieGateContext) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let mut callback = ProbeStoreFlushed::new(context.clone());
    if manager.flush_store(Some(&mut callback)) != 1 {
        delete_probe(context, false);
    }
}

wrap_completion_callback! {
    struct ProbeStoreFlushed {
        context: CookieGateContext,
    }

    impl CompletionCallback {
        fn on_complete(&self) {
            let secure = self
                .context
                .probe_copy()
                .and_then(|probe| {
                    cookie_store_hides_probe(self.context.profile(), probe.as_ref())
                })
                .unwrap_or(false);
            delete_probe(self.context.clone(), secure);
        }
    }
}

fn delete_probe(context: CookieGateContext, secure: bool) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let url = CefString::from(PROBE_COOKIE_URL);
    let name = CefString::from(PROBE_COOKIE_NAME);
    let mut callback = ProbeDeleted::new(context.clone(), secure);
    if manager.delete_cookies(Some(&url), Some(&name), Some(&mut callback)) != 1 {
        context.complete(false);
    }
}

wrap_delete_cookies_callback! {
    struct ProbeDeleted {
        context: CookieGateContext,
        secure: bool,
    }

    impl DeleteCookiesCallback {
        fn on_complete(&self, num_deleted: i32) {
            if num_deleted < 1 {
                self.context.complete(false);
                return;
            }
            flush_cleanup(self.context.clone(), self.secure);
        }
    }
}

fn flush_cleanup(context: CookieGateContext, secure: bool) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let mut callback = CleanupFlushed::new(context.clone(), secure);
    if manager.flush_store(Some(&mut callback)) != 1 {
        context.complete(false);
    }
}

wrap_completion_callback! {
    struct CleanupFlushed {
        context: CookieGateContext,
        secure: bool,
    }

    impl CompletionCallback {
        fn on_complete(&self) {
            self.context.complete(self.secure);
        }
    }
}
