pub fn trim_nul_terminated_string<S: Into<String>>(s: S) -> String {
    let s = s.into();
    let end_index = s.find('\0').unwrap_or(0);
    s[..end_index].to_string()
}
