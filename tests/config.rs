use self_evolve_mcp::config::Config;

#[test]
fn invalid_config_is_rejected() {
    assert!(Config::from_values(Some("not-a-bind"), None).is_err());
    for value in [
        "null",
        "https://example.org/",
        "https://example.org/path",
        "https://user@example.org",
        "https://example.org?x=1",
    ] {
        assert!(Config::from_values(None, Some(value)).is_err(), "{value}");
    }
}

#[test]
fn config_accepts_custom_bind_and_public_origin() {
    let defaults = Config::from_values(None, None).unwrap();
    assert_eq!(defaults.bind.to_string(), "0.0.0.0:8080");
    assert_eq!(defaults.accepted_origin, "https://mcm.hn.yaaha.net");
    let custom = Config::from_values(Some("127.0.0.1:9000"), Some("https://example.org")).unwrap();
    assert_eq!(custom.bind.to_string(), "127.0.0.1:9000");
    assert_eq!(custom.accepted_origin, "https://example.org");
}
