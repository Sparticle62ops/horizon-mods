// More Chat History for Horizon: raises the chat history cap from
// vanilla's 100 entries to 16,384, for the display queue, the send queue
// and the recent-chat list.
//
// A Rust port of MoreChatHistory (red.jackf.jackfredlib and
// red.jackf.morechathistory, JackFred, CC0-1.0,
// https://github.com/JackFredMods/MoreChatHistory). The original's
// behavior is kept:
//
// - The ChatComponent's backing ArrayListDeque is created with capacity
//   16384 instead of vanilla's 100 (upstream's `expandQueue` wrap).
// - The three history-limit constants in addMessageToDisplayQueue,
//   addMessageToQueue and addRecentChat read 16384 instead of 100
//   (upstream's `increaseMaxHistory` modify-expression-value).
// - Every other chat path, trimming and rendering order included, is
//   untouched: only the cap changes.

/// Upstream's expanded history size.
pub const EXPANDED_MAX_HISTORY: usize = 16384;

/// Vanilla's history size, the cap the port replaces.
pub const VANILLA_MAX_HISTORY: usize = 100;

/// The capacity a chat history deque is created with.
pub fn history_capacity() -> usize {
    EXPANDED_MAX_HISTORY
}

/// The cap `add_message_to_display_queue`, `add_message_to_queue` and
/// `add_recent_chat` trim against.
pub fn max_history() -> usize {
    EXPANDED_MAX_HISTORY
}

/// Trim a chat history to the expanded cap, dropping from the oldest
/// side the way ArrayListDeque does.
pub fn trim(history: &mut Vec<String>) {
    if history.len() > EXPANDED_MAX_HISTORY {
        let overflow = history.len() - EXPANDED_MAX_HISTORY;
        history.drain(..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_match_upstream() {
        assert_eq!(history_capacity(), 16384);
        assert_eq!(max_history(), 16384);
        assert_ne!(max_history(), VANILLA_MAX_HISTORY);
    }

    #[test]
    fn trim_drops_oldest_first() {
        let mut history: Vec<String> = (0..16_400).map(|i| format!("msg{i}")).collect();
        trim(&mut history);
        assert_eq!(history.len(), EXPANDED_MAX_HISTORY);
        assert_eq!(history.first(), Some(&"msg16".to_string()));
        assert_eq!(history.last(), Some(&"msg16399".to_string()));
    }

    #[test]
    fn trim_keeps_short_histories() {
        let mut history = vec!["hi".to_string()];
        trim(&mut history);
        assert_eq!(history, vec!["hi".to_string()]);
    }
}
