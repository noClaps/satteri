//! HAST property name → HTML attribute name mapping.
//!
//! Mirrors the attribute-name half of the
//! [`property-information`](https://github.com/wooorm/property-information)
//! package: every known HAST property resolves to its correctly-cased HTML
//! attribute, and unknown properties pass through unchanged.
//!
//! This does not yet carry the richer `Info` data (boolean / overloadedBoolean
//! / spaceSeparated / commaSeparated / booleanish / mustUseProperty); if we
//! need those for encoding-side decisions, the natural next step is a fuller
//! port, potentially as its own crate.

use std::borrow::Cow;

/// Convert a HAST (JS-style) property name to its HTML attribute name, mirroring
/// `property-information` / `hast-util-to-html`.
///
/// Behavior:
/// - Four HTML special cases (className → class, htmlFor → for, httpEquiv → http-equiv,
///   acceptCharset → accept-charset).
/// - All other known HTML properties are ASCII-lowercased (e.g. `srcSet` → `srcset`,
///   `maxLength` → `maxlength`, `onClick` → `onclick`).
/// - ARIA (`ariaFoo` → `aria-foo`), xlink (`xLinkHref` → `xlink:href`),
///   xml (`xmlLang` → `xml:lang`), xmlns (`xmlnsXLink` → `xmlns:xlink`).
/// - data-* uses kebab-case: `dataFooBar` → `data-foo-bar`.
/// - Unknown properties pass through unchanged — matching `property-information`'s
///   behavior for names it has no schema entry for.
pub fn property_to_attribute(name: &str) -> Cow<'_, str> {
    match name {
        "className" => return Cow::Borrowed("class"),
        "htmlFor" => return Cow::Borrowed("for"),
        "httpEquiv" => return Cow::Borrowed("http-equiv"),
        "acceptCharset" => return Cow::Borrowed("accept-charset"),
        "xmlnsXLink" => return Cow::Borrowed("xmlns:xlink"),
        _ => {}
    }

    if is_known_lowercased_html_property(name) {
        return Cow::Owned(name.to_ascii_lowercase());
    }

    if let Some(rest) = strip_namespace_prefix(name, "xLink") {
        return Cow::Owned(format_namespace("xlink:", rest));
    }

    if let Some(rest) = strip_namespace_prefix(name, "xml") {
        return Cow::Owned(format_namespace("xml:", rest));
    }

    // ARIA is intentionally not kebab-cased between words: `ariaValueNow` →
    // `aria-valuenow`, not `aria-value-now`. That's the ARIA spec's convention
    // and differs from the data-* case below.
    if let Some(rest) = strip_namespace_prefix(name, "aria") {
        return Cow::Owned(format_namespace("aria-", rest));
    }

    // data-* *is* kebab-cased: `dataFooBar` → `data-foo-bar`, matching the
    // HTML attribute convention for custom data attributes.
    if let Some(rest) = strip_namespace_prefix(name, "data") {
        return Cow::Owned(format_data_attribute(rest));
    }

    Cow::Borrowed(name)
}

/// Returns the suffix after `prefix` only when the next character is uppercase,
/// so bare words like `datatype` or `arial` don't get namespaced.
fn strip_namespace_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = name.strip_prefix(prefix)?;
    rest.starts_with(|c: char| c.is_ascii_uppercase())
        .then_some(rest)
}

fn format_namespace(prefix: &str, suffix: &str) -> String {
    let mut out = String::with_capacity(prefix.len() + suffix.len());
    out.push_str(prefix);
    for c in suffix.chars() {
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn format_data_attribute(suffix: &str) -> String {
    let mut out = String::with_capacity(4 + suffix.len() + 4);
    out.push_str("data");
    for c in suffix.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Known HTML property names whose HTML attribute form is just the lowercased
/// property name. Mirrors the HTML schema in `property-information` (minus the
/// four special-cased entries handled above). Properties outside this list —
/// including custom/unknown props like `dangerouslySetInnerHTML` — are passed
/// through unchanged, matching `property-information`'s `find()` behavior.
///
/// The list is kept exhaustive rather than heuristic (e.g. "lowercase any
/// camelCase name") so that unknown/custom properties round-trip untouched.
fn is_known_lowercased_html_property(name: &str) -> bool {
    matches!(
        name,
        "accessKey"
            | "allowFullScreen"
            | "allowPaymentRequest"
            | "allowUserMedia"
            | "autoCapitalize"
            | "autoComplete"
            | "autoFocus"
            | "autoPlay"
            | "charSet"
            | "colSpan"
            | "contentEditable"
            | "controlsList"
            | "crossOrigin"
            | "dateTime"
            | "dirName"
            | "encType"
            | "enterKeyHint"
            | "fetchPriority"
            | "formAction"
            | "formEncType"
            | "formMethod"
            | "formNoValidate"
            | "formTarget"
            | "hrefLang"
            | "imageSizes"
            | "imageSrcSet"
            | "inputMode"
            | "isMap"
            | "itemId"
            | "itemProp"
            | "itemRef"
            | "itemScope"
            | "itemType"
            | "maxLength"
            | "minLength"
            | "noModule"
            | "noValidate"
            | "playsInline"
            | "popoverTarget"
            | "popoverTargetAction"
            | "readOnly"
            | "referrerPolicy"
            | "rowSpan"
            | "shadowRootClonable"
            | "shadowRootDelegatesFocus"
            | "shadowRootMode"
            | "spellCheck"
            | "srcDoc"
            | "srcLang"
            | "srcSet"
            | "tabIndex"
            | "typeMustMatch"
            | "useMap"
            | "writingSuggestions"
            | "onAbort"
            | "onAfterPrint"
            | "onAuxClick"
            | "onBeforeMatch"
            | "onBeforePrint"
            | "onBeforeToggle"
            | "onBeforeUnload"
            | "onBlur"
            | "onCancel"
            | "onCanPlay"
            | "onCanPlayThrough"
            | "onChange"
            | "onClick"
            | "onClose"
            | "onContextLost"
            | "onContextMenu"
            | "onContextRestored"
            | "onCopy"
            | "onCueChange"
            | "onCut"
            | "onDblClick"
            | "onDrag"
            | "onDragEnd"
            | "onDragEnter"
            | "onDragExit"
            | "onDragLeave"
            | "onDragOver"
            | "onDragStart"
            | "onDrop"
            | "onDurationChange"
            | "onEmptied"
            | "onEnded"
            | "onError"
            | "onFocus"
            | "onFormData"
            | "onHashChange"
            | "onInput"
            | "onInvalid"
            | "onKeyDown"
            | "onKeyPress"
            | "onKeyUp"
            | "onLanguageChange"
            | "onLoad"
            | "onLoadedData"
            | "onLoadedMetadata"
            | "onLoadEnd"
            | "onLoadStart"
            | "onMessage"
            | "onMessageError"
            | "onMouseDown"
            | "onMouseEnter"
            | "onMouseLeave"
            | "onMouseMove"
            | "onMouseOut"
            | "onMouseOver"
            | "onMouseUp"
            | "onOffline"
            | "onOnline"
            | "onPageHide"
            | "onPageShow"
            | "onPaste"
            | "onPause"
            | "onPlay"
            | "onPlaying"
            | "onPopState"
            | "onProgress"
            | "onRateChange"
            | "onRejectionHandled"
            | "onReset"
            | "onResize"
            | "onScroll"
            | "onScrollEnd"
            | "onSecurityPolicyViolation"
            | "onSeeked"
            | "onSeeking"
            | "onSelect"
            | "onSlotChange"
            | "onStalled"
            | "onStorage"
            | "onSubmit"
            | "onSuspend"
            | "onTimeUpdate"
            | "onToggle"
            | "onUnhandledRejection"
            | "onUnload"
            | "onVolumeChange"
            | "onWaiting"
            | "onWheel"
            | "aLink"
            | "bgColor"
            | "borderColor"
            | "bottomMargin"
            | "cellPadding"
            | "cellSpacing"
            | "charOff"
            | "classId"
            | "codeBase"
            | "codeType"
            | "frameBorder"
            | "hSpace"
            | "leftMargin"
            | "longDesc"
            | "lowSrc"
            | "marginHeight"
            | "marginWidth"
            | "noHref"
            | "noResize"
            | "noShade"
            | "noWrap"
            | "rightMargin"
            | "topMargin"
            | "vAlign"
            | "vLink"
            | "vSpace"
            | "valueType"
            | "allowTransparency"
            | "autoCorrect"
            | "autoSave"
            | "disablePictureInPicture"
            | "disableRemotePlayback"
    )
}

#[cfg(test)]
mod tests {
    use super::property_to_attribute;

    #[test]
    fn html_special_cases() {
        assert_eq!(property_to_attribute("className"), "class");
        assert_eq!(property_to_attribute("htmlFor"), "for");
        assert_eq!(property_to_attribute("httpEquiv"), "http-equiv");
        assert_eq!(property_to_attribute("acceptCharset"), "accept-charset");
    }

    #[test]
    fn known_html_properties_are_lowercased() {
        assert_eq!(property_to_attribute("srcSet"), "srcset");
        assert_eq!(property_to_attribute("maxLength"), "maxlength");
        assert_eq!(property_to_attribute("minLength"), "minlength");
        assert_eq!(property_to_attribute("readOnly"), "readonly");
        assert_eq!(property_to_attribute("autoPlay"), "autoplay");
        assert_eq!(property_to_attribute("autoFocus"), "autofocus");
        assert_eq!(property_to_attribute("contentEditable"), "contenteditable");
        assert_eq!(property_to_attribute("tabIndex"), "tabindex");
        assert_eq!(property_to_attribute("colSpan"), "colspan");
        assert_eq!(property_to_attribute("rowSpan"), "rowspan");
        assert_eq!(property_to_attribute("crossOrigin"), "crossorigin");
        assert_eq!(property_to_attribute("dateTime"), "datetime");
        assert_eq!(property_to_attribute("charSet"), "charset");
        assert_eq!(property_to_attribute("noValidate"), "novalidate");
        assert_eq!(property_to_attribute("referrerPolicy"), "referrerpolicy");
        assert_eq!(property_to_attribute("inputMode"), "inputmode");
        assert_eq!(property_to_attribute("enterKeyHint"), "enterkeyhint");
        assert_eq!(property_to_attribute("spellCheck"), "spellcheck");
        assert_eq!(property_to_attribute("accessKey"), "accesskey");
        assert_eq!(property_to_attribute("itemProp"), "itemprop");
        assert_eq!(property_to_attribute("imageSrcSet"), "imagesrcset");
        assert_eq!(property_to_attribute("formNoValidate"), "formnovalidate");
    }

    #[test]
    fn event_handlers_are_lowercased() {
        assert_eq!(property_to_attribute("onClick"), "onclick");
        assert_eq!(property_to_attribute("onKeyDown"), "onkeydown");
        assert_eq!(property_to_attribute("onMouseOver"), "onmouseover");
        assert_eq!(
            property_to_attribute("onCanPlayThrough"),
            "oncanplaythrough"
        );
    }

    #[test]
    fn legacy_properties_are_lowercased() {
        assert_eq!(property_to_attribute("bgColor"), "bgcolor");
        assert_eq!(property_to_attribute("cellPadding"), "cellpadding");
        assert_eq!(property_to_attribute("vAlign"), "valign");
        assert_eq!(property_to_attribute("longDesc"), "longdesc");
    }

    #[test]
    fn aria_lowercases_suffix_without_inner_hyphens() {
        assert_eq!(property_to_attribute("ariaHidden"), "aria-hidden");
        assert_eq!(property_to_attribute("ariaLive"), "aria-live");
        // ARIA attributes do NOT get inner hyphens between words.
        assert_eq!(property_to_attribute("ariaValueNow"), "aria-valuenow");
        assert_eq!(
            property_to_attribute("ariaActiveDescendant"),
            "aria-activedescendant"
        );
    }

    #[test]
    fn data_kebab_cases_suffix() {
        assert_eq!(property_to_attribute("dataLanguage"), "data-language");
        assert_eq!(property_to_attribute("dataFooBar"), "data-foo-bar");
    }

    #[test]
    fn xlink_namespaces_lowercased_suffix() {
        assert_eq!(property_to_attribute("xLinkHref"), "xlink:href");
        assert_eq!(property_to_attribute("xLinkActuate"), "xlink:actuate");
        assert_eq!(property_to_attribute("xLinkArcRole"), "xlink:arcrole");
        assert_eq!(property_to_attribute("xLinkType"), "xlink:type");
    }

    #[test]
    fn xml_namespaces_lowercased_suffix() {
        assert_eq!(property_to_attribute("xmlLang"), "xml:lang");
        assert_eq!(property_to_attribute("xmlBase"), "xml:base");
        assert_eq!(property_to_attribute("xmlSpace"), "xml:space");
    }

    #[test]
    fn xmlns_special_cases() {
        assert_eq!(property_to_attribute("xmlnsXLink"), "xmlns:xlink");
    }

    #[test]
    fn unknown_properties_pass_through() {
        assert_eq!(property_to_attribute("foo"), "foo");
        assert_eq!(property_to_attribute("my-custom"), "my-custom");
        // Property that does not start with an uppercase after the prefix is unchanged.
        assert_eq!(property_to_attribute("datatype"), "datatype");
        assert_eq!(property_to_attribute("arial"), "arial");
        // Custom/React-style properties unknown to property-information pass through.
        assert_eq!(
            property_to_attribute("dangerouslySetInnerHTML"),
            "dangerouslySetInnerHTML"
        );
        assert_eq!(property_to_attribute("customProp"), "customProp");
    }
}
