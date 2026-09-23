# MoveDock localization

Approved scope: localize MoveDock UI, notifications, confirmations and application errors. Preserve external Wordmove/SSH/rsync output. Match YTDown's eight locales: ja, en, zh-TW, fr, es, pt-BR, de, ko.

Use Vue I18n with separate, complete JSON catalogs and English fallback. A settings selector applies immediately and persists independently of execution settings, so changing language does not invalidate trusted environments or discard unsaved connection settings. Initial locale follows the system/browser language, with the same regional mapping as YTDown. Update the document language and format history dates with the active locale.

Application messages from Rust carry a stable code, parameters and optional original details in a versioned envelope inside the existing string IPC/storage fields. The frontend translates only those envelopes; raw subprocess output stays unchanged. Legacy persisted setup notes are mapped to message codes when displayed. Unknown errors remain visible verbatim. Common connection errors get a localized explanatory hint without hiding their original details.

Keep all existing trust, overwrite, cancellation and dirty-file safeguards. Translate whole sentences with named placeholders. Do not translate filenames, environment identifiers, Movefile content or CLI arguments. Layout wraps long translations without reducing existing text sizes. Language names are shown in their native spelling.

Validation: all eight catalogs have identical keys and placeholders; locale detection/persistence/fallback tests; localized confirmations and live language switching tests; original-log and error-detail preservation tests; existing Vue and Rust suites; production frontend and universal macOS builds; inspect the welcome, settings, sync, editor, history and log UI at 1180 and 900 pixels. Install the tested universal app in the user's Applications folder. README and release notes remain bilingual Japanese/English. No live site synchronization is required for this change.

Implementation sequence: catalogs and locale service; UI/dialog conversion; stable backend message codes and legacy notes; tests and layout checks; documentation, universal build and installation.
