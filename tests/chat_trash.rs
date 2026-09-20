use wordweave5::{chat::{self, Conversation}, store::Progress};

fn deleted(mut chat: Conversation) -> Conversation {
    chat.draft = "Explain this word".into();
    let mut value = serde_json::to_value(chat).unwrap();
    value["deleted_at"] = serde_json::json!(123);
    serde_json::from_value(value).unwrap()
}

#[test]
fn trash_is_hidden_and_cannot_be_sent_but_keeps_the_draft() {
    let archived = deleted(Conversation::new());
    assert!(chat::ordered_indices(&[archived.clone()]).is_empty());
    assert!(chat::prepare(&archived).is_err());
    assert_eq!(archived.draft, "Explain this word");
    assert_eq!(serde_json::to_value(&archived).unwrap()["deleted_at"], 123);
}

#[test]
fn trash_does_not_use_the_fifty_active_conversation_slots() {
    let mut progress = Progress::default();
    progress.chats = (0..50).map(|_| Conversation::new()).collect();
    progress.chats.push(deleted(Conversation::new()));
    assert!(progress.validate().is_ok());
    progress.chats.push(Conversation::new());
    assert!(progress.validate().is_err());
}
