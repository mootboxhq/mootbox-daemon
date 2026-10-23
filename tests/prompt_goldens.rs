use mootbox_daemon::contracts::{ClaimResponse, ClaimResponseClaimedRunManifest};
use mootbox_daemon::run::prompt;

const CHAT_CONTEXT_HEADING: &str = "Chat context (set by the people in this chat";

fn fixture_manifest(fixture: &str) -> Option<ClaimResponseClaimedRunManifest> {
    match serde_json::from_str(fixture).ok()? {
        ClaimResponse::Claimed { run } => Some(run.manifest),
        _ => None,
    }
}

#[test]
fn shared_chat_prompt_has_history_and_no_task_or_delegation_flow() {
    let manifest = fixture_manifest(include_str!(
        "../contracts/runner/fixtures/claim-response.claimed-execute.json"
    ))
    .expect("claimed fixture");
    let text = prompt::execute_prompt(&manifest).expect("prompt renders");
    assert!(text.contains("Frontend needs name and email"));
    assert!(text.contains("Do not automatically commit, merge, push, or deploy"));
    assert!(text.contains("mootbox_post_chat_message"));
    assert!(text.contains("mootbox_submit_result"));
    assert!(text.contains("call mootbox_ask_question"));
    assert!(text.contains(
        "Before any push, merge, deploy, or destructive or irreversible command, call mootbox_request_approval"
    ));
    assert!(text.contains("call mootbox_submit_result with a short summary and stop"));
    assert!(text.contains(
        "When the work's state changes meaningfully, call mootbox_update_summary with a short summary and next steps."
    ));
    assert!(!text.contains("mootbox_create_handoff"));
}

#[test]
fn prompt_without_chat_context_omits_the_section() {
    let manifest = fixture_manifest(include_str!(
        "../contracts/runner/fixtures/claim-response.claimed-execute.json"
    ))
    .expect("claimed fixture");
    let text = prompt::execute_prompt(&manifest).expect("prompt renders");
    assert!(!text.contains(CHAT_CONTEXT_HEADING));
    assert!(!text.contains("Pinned files to read first"));
    assert!(!text.contains("Linked chats"));
}

#[test]
fn prompt_with_chat_context_lists_instructions_pinned_files_and_linked_chats_before_the_messages() {
    let manifest = fixture_manifest(include_str!(
        "../contracts/runner/fixtures/claim-response.claimed-execute.with-context.json"
    ))
    .expect("claimed fixture");
    let text = prompt::execute_prompt(&manifest).expect("prompt renders");
    let section = text.find(CHAT_CONTEXT_HEADING).expect("context section");
    let messages = text.find("Messages through this submission").expect("messages section");
    assert!(section < messages);
    assert!(text.contains("Description: Let people find users by name or email from the admin search box."));
    assert!(text.contains(
        "Instructions: Keep the users endpoint backwards compatible. Run pnpm check before saying a change is done."
    ));
    assert!(text.contains("Pinned files to read first: apps/api/src/routes/users.ts, docs/search.md"));
    assert!(text.contains("Linked chats: Admin search box (Frontend)"));
}
