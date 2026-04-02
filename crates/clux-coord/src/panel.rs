use std::sync::Arc;

use crate::broker::Broker;
use crate::protocol::{PeerInfo, PeerMessage, TaskRequest};

/// Data model for the coordination panel overlay.
#[derive(Debug, Default)]
pub struct CoordPanelData {
    pub peers: Vec<PeerInfo>,
    pub recent_messages: Vec<PeerMessage>,
    pub tasks: Vec<TaskRequest>,
}

/// Manages the coordination panel state.
pub struct CoordPanel {
    broker: Arc<Broker>,
    /// Whether the panel is currently visible.
    pub visible: bool,
    /// Cached panel data, refreshed on toggle or periodically.
    pub data: CoordPanelData,
}

impl CoordPanel {
    pub fn new(broker: Arc<Broker>) -> Self {
        Self {
            broker,
            visible: false,
            data: CoordPanelData::default(),
        }
    }

    /// Toggle panel visibility. Refreshes data when opening.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.refresh();
        }
    }

    /// Refresh panel data from the broker.
    pub fn refresh(&mut self) {
        self.data.peers = self.broker.list_peers(None).unwrap_or_default();
        // Read recent broadcast messages (last 50)
        self.data.recent_messages = self
            .broker
            .read_messages("__panel__", None)
            .unwrap_or_default()
            .into_iter()
            .rev()
            .take(50)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        self.data.tasks = self.broker.list_tasks(None).unwrap_or_default();
    }

    /// Render the panel as a text block for display.
    /// Returns lines of text to render as an overlay.
    pub fn render_text(&self, max_width: usize) -> Vec<String> {
        if !self.visible {
            return Vec::new();
        }

        let mut lines = Vec::new();
        let separator: String = "\u{2500}".repeat(max_width.min(60));

        lines.push(format!("\u{250c}{separator}\u{2510}"));
        lines.push(format!(
            "\u{2502}{:^w$}\u{2502}",
            "Coordination Panel",
            w = separator.len()
        ));
        lines.push(format!("\u{251c}{separator}\u{2524}"));

        // Peers section
        lines.push(format!(
            "\u{2502}{:^w$}\u{2502}",
            "Peers",
            w = separator.len()
        ));
        lines.push(format!("\u{251c}{separator}\u{2524}"));
        if self.data.peers.is_empty() {
            lines.push(format!(
                "\u{2502}{:<w$}\u{2502}",
                " No active peers",
                w = separator.len()
            ));
        } else {
            for peer in &self.data.peers {
                let status = peer.status_text.as_deref().unwrap_or("idle");
                let line = format!(" {} (pane {}) - {}", peer.peer_id, peer.pane_id, status);
                let truncated = if line.len() > separator.len() {
                    format!("{}\u{2026}", &line[..separator.len() - 1])
                } else {
                    line
                };
                lines.push(format!(
                    "\u{2502}{:<w$}\u{2502}",
                    truncated,
                    w = separator.len()
                ));
            }
        }

        // Tasks section
        lines.push(format!("\u{251c}{separator}\u{2524}"));
        lines.push(format!(
            "\u{2502}{:^w$}\u{2502}",
            "Tasks",
            w = separator.len()
        ));
        lines.push(format!("\u{251c}{separator}\u{2524}"));
        if self.data.tasks.is_empty() {
            lines.push(format!(
                "\u{2502}{:<w$}\u{2502}",
                " No tasks",
                w = separator.len()
            ));
        } else {
            for task in &self.data.tasks {
                let line = format!(" [{}] {}", task.status, task.description);
                let truncated = if line.len() > separator.len() {
                    format!("{}\u{2026}", &line[..separator.len() - 1])
                } else {
                    line
                };
                lines.push(format!(
                    "\u{2502}{:<w$}\u{2502}",
                    truncated,
                    w = separator.len()
                ));
            }
        }

        // Messages section
        lines.push(format!("\u{251c}{separator}\u{2524}"));
        lines.push(format!(
            "\u{2502}{:^w$}\u{2502}",
            "Recent Messages",
            w = separator.len()
        ));
        lines.push(format!("\u{251c}{separator}\u{2524}"));
        let recent: Vec<_> = self.data.recent_messages.iter().rev().take(10).collect();
        if recent.is_empty() {
            lines.push(format!(
                "\u{2502}{:<w$}\u{2502}",
                " No messages",
                w = separator.len()
            ));
        } else {
            for msg in recent.iter().rev() {
                let to = msg.to_peer.as_deref().unwrap_or("all");
                let body_preview = msg.body.to_string();
                let line = format!(" {} \u{2192} {}: {}", msg.from_peer, to, body_preview);
                let truncated = if line.len() > separator.len() {
                    format!("{}\u{2026}", &line[..separator.len() - 1])
                } else {
                    line
                };
                lines.push(format!(
                    "\u{2502}{:<w$}\u{2502}",
                    truncated,
                    w = separator.len()
                ));
            }
        }

        lines.push(format!("\u{2514}{separator}\u{2518}"));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_panel() -> CoordPanel {
        let broker = Arc::new(Broker::in_memory().unwrap());
        CoordPanel::new(broker)
    }

    #[test]
    fn panel_starts_hidden() {
        let panel = test_panel();
        assert!(!panel.visible);
        assert!(panel.render_text(60).is_empty());
    }

    #[test]
    fn toggle_shows_and_hides() {
        let mut panel = test_panel();
        panel.toggle();
        assert!(panel.visible);
        assert!(!panel.render_text(60).is_empty());
        panel.toggle();
        assert!(!panel.visible);
        assert!(panel.render_text(60).is_empty());
    }

    #[test]
    fn panel_shows_peers() {
        let broker = Arc::new(Broker::in_memory().unwrap());
        broker.register_peer("p1", 0).unwrap();
        broker.set_status("p1", "testing").unwrap();

        let mut panel = CoordPanel::new(broker);
        panel.toggle();

        let text = panel.render_text(60).join("\n");
        assert!(text.contains("p1"));
        assert!(text.contains("testing"));
    }

    #[test]
    fn panel_shows_tasks() {
        let broker = Arc::new(Broker::in_memory().unwrap());
        let task = crate::protocol::TaskRequest {
            id: None,
            description: "run tests".into(),
            status: crate::protocol::TaskStatus::Pending,
            requester: "a".into(),
            assignee: None,
            created_at: None,
        };
        broker.create_task(&task).unwrap();

        let mut panel = CoordPanel::new(broker);
        panel.toggle();

        let text = panel.render_text(60).join("\n");
        assert!(text.contains("run tests"));
        assert!(text.contains("pending"));
    }
}
