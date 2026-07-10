use tokio::sync::watch;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SubagentTerminalState {
    pub sequence: u64,
    pub report_persistence_failed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentTerminalKind {
    ReportPersisted,
    ReportPersistenceFailed,
}

#[derive(Clone)]
pub struct SubagentTerminalNotifier {
    sender: watch::Sender<SubagentTerminalState>,
}

impl SubagentTerminalNotifier {
    pub fn notify(&self, kind: SubagentTerminalKind) {
        self.sender.send_modify(|state| {
            state.sequence = state.sequence.saturating_add(1);
            state.report_persistence_failed |=
                kind == SubagentTerminalKind::ReportPersistenceFailed;
        });
    }

    pub fn subscribe(&self) -> watch::Receiver<SubagentTerminalState> {
        self.sender.subscribe()
    }
}

pub fn notifier() -> SubagentTerminalNotifier {
    let (sender, _) = watch::channel(SubagentTerminalState::default());
    SubagentTerminalNotifier { sender }
}
