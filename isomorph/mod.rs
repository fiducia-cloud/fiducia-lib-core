//! Code that runs identically everywhere: pure domain logic, validation, ids, redaction.

/// Redact anything that looks like a bearer token, PAT, or connection-string password.
pub fn redact(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for word in input.split_inclusive(char::is_whitespace) {
        let trimmed = word.trim_end();
        let suffix = &word[trimmed.len()..];
        let looks_secret = trimmed.starts_with("ghp_")
            || trimmed.starts_with("github_pat_")
            || trimmed.starts_with("lin_api_")
            || trimmed.starts_with("Bearer ")
            || (trimmed.contains("://") && trimmed.contains('@') && trimmed.contains(':'));
        if looks_secret {
            out.push_str("<redacted>");
        } else {
            out.push_str(trimmed);
        }
        out.push_str(suffix);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn redacts_tokens_and_urls_with_passwords() {
        let s = super::redact("token ghp_abc123 url postgres://u:p@h/db ok");
        assert_eq!(s, "token <redacted> url <redacted> ok");
    }
}
