pub const ABSTRACT_ROLES: &[&str] = &[
    "command",
    "composite",
    "input",
    "landmark",
    "range",
    "roletype",
    "section",
    "sectionhead",
    "select",
    "structure",
    "widget",
    "window",
];

pub fn is_abstract_role(role: &str) -> bool {
    split_roles(role)
        .any(|token| ABSTRACT_ROLES.iter().any(|known| known.eq_ignore_ascii_case(token)))
}

pub fn split_roles(role: &str) -> impl Iterator<Item = &str> {
    role.split(|ch: char| ch.is_whitespace() || ch == ',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

#[cfg(test)]
mod tests {
    use super::is_abstract_role;

    #[test]
    fn known_abstract_role_is_detected() {
        assert!(is_abstract_role("widget"));
        assert!(is_abstract_role("landmark"));
        assert!(is_abstract_role("Widget"));
        assert!(is_abstract_role("button widget"));
    }

    #[test]
    fn concrete_or_unknown_roles_are_rejected() {
        assert!(!is_abstract_role("button"));
        assert!(!is_abstract_role("dialog"));
        assert!(!is_abstract_role("made-up-role"));
    }
}
