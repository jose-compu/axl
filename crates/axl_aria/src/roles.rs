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
    ABSTRACT_ROLES.contains(&role)
}

#[cfg(test)]
mod tests {
    use super::is_abstract_role;

    #[test]
    fn known_abstract_role_is_detected() {
        assert!(is_abstract_role("widget"));
        assert!(is_abstract_role("landmark"));
    }

    #[test]
    fn concrete_or_unknown_roles_are_rejected() {
        assert!(!is_abstract_role("button"));
        assert!(!is_abstract_role("dialog"));
        assert!(!is_abstract_role("made-up-role"));
    }
}
