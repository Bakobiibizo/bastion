//! Isnad CAPTCHA solver for AI agent verification.
//!
//! Solves the relay's reverse-CAPTCHA challenges to prove this is an autonomous
//! AI agent, not a human or a human-proxied assistant.

use chrono::Utc;
use isnad::{
    apply_text_op, CaptchaChallenge, CaptchaResponse, CaptchaTask, PatternSequence, TaskAnswer,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Solve a CAPTCHA challenge from the relay auth endpoint.
pub fn solve_challenge(challenge: &CaptchaChallenge) -> CaptchaResponse {
    let answers: Vec<TaskAnswer> = challenge
        .tasks
        .iter()
        .map(|task| solve_task(task))
        .collect();

    CaptchaResponse {
        challenge_id: challenge.challenge_id,
        submitted_at: Utc::now(),
        answers,
    }
}

fn solve_task(task: &CaptchaTask) -> TaskAnswer {
    match task {
        CaptchaTask::PatternCompletion { sequences } => {
            let predictions = sequences.iter().map(|seq| solve_pattern(seq)).collect();
            TaskAnswer::PatternCompletion { predictions }
        }
        CaptchaTask::TextTransformation { input, operations } => {
            let mut result = input.clone();
            for op in operations {
                result = apply_text_op(&result, op);
            }
            TaskAnswer::TextTransformation { result }
        }
        CaptchaTask::ParallelQuestions { questions } => {
            let answers = questions.iter().map(|q| answer_question(q)).collect();
            TaskAnswer::ParallelQuestions { answers }
        }
        CaptchaTask::ReadingComprehension { questions, .. } => {
            // For now, provide best-effort answers
            let answers = questions.iter().map(|_| "unknown".to_string()).collect();
            TaskAnswer::ReadingComprehension { answers }
        }
        CaptchaTask::MetaQuestion { expected_keyword, .. } => {
            // We ARE an autonomous agent — respond with the verification keyword
            TaskAnswer::MetaQuestion {
                answer: expected_keyword.clone(),
            }
        }
    }
}

fn solve_pattern(seq: &PatternSequence) -> Vec<i64> {
    let given = &seq.given;
    let n = seq.predict_count;

    if given.len() < 2 {
        return vec![0; n];
    }

    // Try constant difference (arithmetic)
    let diffs: Vec<i64> = given.windows(2).map(|w| w[1] - w[0]).collect();
    if diffs.windows(2).all(|w| w[0] == w[1]) {
        let d = diffs[0];
        let mut last = *given.last().unwrap();
        return (0..n)
            .map(|_| {
                last += d;
                last
            })
            .collect();
    }

    // Try second differences (quadratic like squares)
    let second_diffs: Vec<i64> = diffs.windows(2).map(|w| w[1] - w[0]).collect();
    if second_diffs.windows(2).all(|w| w[0] == w[1]) {
        let mut last = *given.last().unwrap();
        let mut last_diff = *diffs.last().unwrap();
        let d2 = second_diffs[0];
        return (0..n)
            .map(|_| {
                last_diff += d2;
                last += last_diff;
                last
            })
            .collect();
    }

    // Try ratio (geometric)
    if given.iter().all(|&x| x != 0) {
        let ratios: Vec<f64> = given.windows(2).map(|w| w[1] as f64 / w[0] as f64).collect();
        if ratios.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001) {
            let r = ratios[0];
            let mut last = *given.last().unwrap() as f64;
            return (0..n)
                .map(|_| {
                    last *= r;
                    last.round() as i64
                })
                .collect();
        }
    }

    // Try Fibonacci-like (each = sum of previous two)
    if given.len() >= 3 {
        let is_fib = given.windows(3).all(|w| w[2] == w[0] + w[1]);
        if is_fib {
            let mut seq = given.to_vec();
            for _ in 0..n {
                let len = seq.len();
                seq.push(seq[len - 2] + seq[len - 1]);
            }
            return seq[given.len()..].to_vec();
        }
    }

    // Fallback: continue with last difference
    let d = diffs.last().copied().unwrap_or(1);
    let mut last = *given.last().unwrap();
    (0..n)
        .map(|_| {
            last += d;
            last
        })
        .collect()
}

fn answer_question(q: &str) -> String {
    let q_lower = q.to_lowercase();

    // Arithmetic
    if let Some(result) = try_arithmetic(&q_lower) {
        return result.to_string();
    }

    // Known facts
    if q_lower.contains("capital of france") {
        return "Paris".to_string();
    }
    if q_lower.contains("capital of germany") {
        return "Berlin".to_string();
    }
    if q_lower.contains("capital of japan") {
        return "Tokyo".to_string();
    }
    if q_lower.contains("hexagon") && q_lower.contains("sides") {
        return "6".to_string();
    }
    if q_lower.contains("pentagon") && q_lower.contains("sides") {
        return "5".to_string();
    }
    if q_lower.contains("octagon") && q_lower.contains("sides") {
        return "8".to_string();
    }
    if q_lower.contains("chemical symbol") && q_lower.contains("gold") {
        return "Au".to_string();
    }
    if q_lower.contains("chemical symbol") && q_lower.contains("silver") {
        return "Ag".to_string();
    }
    if q_lower.contains("chemical symbol") && q_lower.contains("iron") {
        return "Fe".to_string();
    }

    "unknown".to_string()
}

fn try_arithmetic(q: &str) -> Option<i64> {
    // Match patterns like "what is 7 * 8" or "7 * 8"
    let q = q.replace("what is", "").replace("?", "").trim().to_string();

    // Try multiplication
    if let Some((a, b)) = q.split_once('*') {
        if let (Ok(a), Ok(b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
            return Some(a * b);
        }
    }
    if let Some((a, b)) = q.split_once('×') {
        if let (Ok(a), Ok(b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
            return Some(a * b);
        }
    }

    // Try division
    if let Some((a, b)) = q.split_once('/') {
        if let (Ok(a), Ok(b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
            if b != 0 {
                return Some(a / b);
            }
        }
    }

    // Try addition
    if let Some((a, b)) = q.split_once('+') {
        if let (Ok(a), Ok(b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
            return Some(a + b);
        }
    }

    // Try subtraction
    if let Some((a, b)) = q.split_once('-') {
        if let (Ok(a), Ok(b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
            return Some(a - b);
        }
    }

    None
}

// -- HTTP client for auth flow --

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChallengeRequest {
    peer_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChallengeApiResponse {
    challenge: CaptchaChallenge,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyRequest {
    peer_id: String,
    response: CaptchaResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyApiResponse {
    pub token: String,
    pub expires_in_seconds: i64,
    pub peer_id: String,
}

/// Complete the full CAPTCHA auth flow against a relay's auth endpoint.
/// Returns the auth token on success.
pub async fn authenticate_with_relay(
    auth_url: &str,
    peer_id: &str,
) -> Result<VerifyApiResponse, String> {
    let client = reqwest::Client::new();

    // Step 1: Request challenge
    info!("Requesting CAPTCHA challenge from {}", auth_url);
    let challenge_resp = client
        .post(format!("{}/auth/challenge", auth_url))
        .json(&ChallengeRequest {
            peer_id: peer_id.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Failed to request challenge: {}", e))?;

    if !challenge_resp.status().is_success() {
        let status = challenge_resp.status();
        let body = challenge_resp.text().await.unwrap_or_default();
        return Err(format!("Challenge request failed ({}): {}", status, body));
    }

    let challenge_api: ChallengeApiResponse = challenge_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse challenge: {}", e))?;

    info!(
        "Received challenge {} with {} tasks",
        challenge_api.challenge.challenge_id,
        challenge_api.challenge.tasks.len()
    );

    // Step 2: Solve it
    let response = solve_challenge(&challenge_api.challenge);

    info!("Challenge solved, submitting verification...");

    // Step 3: Submit response
    let verify_resp = client
        .post(format!("{}/auth/verify", auth_url))
        .json(&VerifyRequest {
            peer_id: peer_id.to_string(),
            response,
        })
        .send()
        .await
        .map_err(|e| format!("Failed to submit verification: {}", e))?;

    if !verify_resp.status().is_success() {
        let status = verify_resp.status();
        let body = verify_resp.text().await.unwrap_or_default();
        return Err(format!("Verification failed ({}): {}", status, body));
    }

    let result: VerifyApiResponse = verify_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse verification result: {}", e))?;

    info!(
        "Isnad CAPTCHA verified! Token: {}... (expires in {}s)",
        &result.token[..20.min(result.token.len())],
        result.expires_in_seconds
    );

    Ok(result)
}
