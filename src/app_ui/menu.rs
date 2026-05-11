use crate::settings::Settings;
use ledger_device_sdk::include_gif;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::nbgl::{NbglGlyph, NbglHomeAndSettings};

pub fn ui_menu_main(_: &mut Comm) -> NbglHomeAndSettings {
    #[cfg(target_os = "apex_p")]
    const APP_ICON: NbglGlyph =
        NbglGlyph::from_include(include_gif!("icons/squads_32x32.png", NBGL));
    #[cfg(target_os = "stax")]
    const APP_ICON: NbglGlyph =
        NbglGlyph::from_include(include_gif!("icons/squads_32x32.gif", NBGL));
    #[cfg(target_os = "flex")]
    const APP_ICON: NbglGlyph =
        NbglGlyph::from_include(include_gif!("icons/squads_40x40.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const APP_ICON: NbglGlyph =
        NbglGlyph::from_include(include_gif!("icons/squads_14x14.gif", NBGL));

    let settings_strings = [["Blind signing", "Allow signing unrecognized transactions."]];
    let mut settings: Settings = Default::default();

    NbglHomeAndSettings::new()
        .glyph(&APP_ICON)
        .settings(settings.get_mut(), &settings_strings)
        .infos(
            "Squads",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
        )
}
