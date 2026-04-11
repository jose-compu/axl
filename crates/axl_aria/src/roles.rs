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
