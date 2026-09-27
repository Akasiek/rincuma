pub(super) fn validate_hex_color(color: &str, _: &()) -> garde::Result {
    if color
        .strip_prefix('#')
        .is_some_and(|hex| hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        Ok(())
    } else {
        Err(garde::Error::new(
            "color must be a hex value in #RRGGBB format",
        ))
    }
}
