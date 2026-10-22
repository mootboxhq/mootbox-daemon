use mootbox_daemon::contracts::ClaimResponse;
use mootbox_daemon::run::prompt;

#[test]
fn shared_chat_prompt_has_history_and_no_task_or_delegation_flow() {
    let fixture = include_str!("../contracts/runner/fixtures/claim-response.claimed-execute.json");
    let ClaimResponse::Claimed { run } = serde_json::from_str(fixture).unwrap() else {
        panic!("claim");
    };
    let text = prompt::execute_prompt(&run.manifest).expect("prompt renders");
    assert!(text.contains("Frontend needs name and email"));
    assert!(text.contains("Do not automatically commit, merge, push, or deploy"));
    assert!(text.contains("mootbox_post_chat_message"));
    assert!(text.contains("mootbox_submit_result"));
    assert!(!text.contains("mootbox_create_handoff"));
}
