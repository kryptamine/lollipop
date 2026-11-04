use objc2_foundation::NSLocale;

/// The first language in System Settings' preferred list, such as `ru-RU`.
pub(crate) fn system_locale() -> Option<String> {
    NSLocale::preferredLanguages()
        .firstObject()
        .map(|language| language.to_string())
}
