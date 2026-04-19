pub mod pty_session;

use pty_session::PtySession;
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<u32, PtySession>>>,
}

#[derive(Clone, serde::Serialize)]
struct PtyOutputEvent {
    id: u32,
    data: String,
}

#[derive(Clone, serde::Serialize)]
struct PtyExitEvent {
    id: u32,
    code: u32,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn spawn(
        &self,
        app: &AppHandle,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<u32, String> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let (session, reader) = PtySession::spawn(cwd, cols, rows)?;

        self.sessions
            .lock()
            .map_err(|e| format!("lock: {}", e))?
            .insert(id, session);

        self.start_reader_thread(id, reader, app.clone());

        Ok(id)
    }

    pub fn write(&self, id: u32, data: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("lock: {}", e))?;
        let session = sessions
            .get(&id)
            .ok_or_else(|| format!("pty {} not found", id))?;
        session.write(data)
    }

    pub fn resize(&self, id: u32, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("lock: {}", e))?;
        let session = sessions
            .get(&id)
            .ok_or_else(|| format!("pty {} not found", id))?;
        session.resize(cols, rows)
    }

    pub fn kill(&self, id: u32) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("lock: {}", e))?;
        let mut session = sessions
            .remove(&id)
            .ok_or_else(|| format!("pty {} not found", id))?;
        session.kill()
    }

    pub fn kill_all(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for (_, mut session) in sessions.drain() {
                let _ = session.kill();
            }
        }
    }

    fn start_reader_thread(
        &self,
        id: u32,
        mut reader: Box<dyn Read + Send>,
        app: AppHandle,
    ) {
        let sessions = Arc::clone(&self.sessions);

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app.emit("pty-output", PtyOutputEvent { id, data: text });
                    }
                    Err(_) => break,
                }
            }

            let code = sessions
                .lock()
                .ok()
                .and_then(|mut s| s.remove(&id))
                .and_then(|mut s| s.try_wait())
                .unwrap_or(0);

            let _ = app.emit("pty-exit", PtyExitEvent { id, code });
        });
    }
}
