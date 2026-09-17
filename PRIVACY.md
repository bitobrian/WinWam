# WinWam Privacy Policy

**Effective date:** September 17, 2026

WinWam is a Windows application for browsing and managing World of Warcraft addons. This policy describes how the current app handles information and is intended to give you a clear notice of those practices, including under the EU/UK GDPR, the California Consumer Privacy Act as amended by the CPRA (CCPA), and Brazil’s LGPD.

WinWam is pre-1.0; material changes to these practices will be reflected here before they ship. This policy is not legal advice.

## Summary

WinWam does not require an account, display advertising, sell or share personal information, or send analytics, crash reports, or in-app telemetry to the developer.

Settings, addon metadata, and diagnostic files stay on your device. The only automatic network use today is fetching a public addon-directory JSON file over HTTPS. The developer does not receive a copy of that request and does not maintain user accounts or a user database.

## Who we are

WinWam is an independent open-source project published at [github.com/bitobrian/WinWam](https://github.com/bitobrian/WinWam). It is not affiliated with Blizzard Entertainment or Microsoft.

Where a privacy law asks for a “controller,” “business,” or “controlador,” that role is the independent developer of WinWam. No data protection officer (DPO / *encarregado*) is appointed, because the developer does not operate a service that stores personal data about users.

For privacy questions or requests, open an issue in the [WinWam GitHub repository](https://github.com/bitobrian/WinWam/issues). GitHub issues are public. Do not paste logs, folder paths, or other details you do not want published.

## What WinWam does not collect or access

WinWam does not:

- Create accounts or log into Battle.net, Blizzard, GitHub, or any other service on your behalf.
- Read World of Warcraft account, character, chat, screenshot, or `WTF` / SavedVariables data.
- Scan your disk. Folder detection checks a short list of standard `World of Warcraft` install paths.
- Use advertising identifiers, cookies, or third-party analytics or crash-reporting SDKs.
- Sell, rent, or share personal information, including for cross-context behavioral advertising.
- Make automated decisions that produce legal or similarly significant effects.

## Information processed on your device

WinWam may access and process the following locally to provide its features:

- The World of Warcraft installation folder you select, or one detected at a standard install path.
- Addon folders under that installation’s `Interface\AddOns` directory, including `.toc` metadata (title, version, interface, author) and any `.winwam-id` marker WinWam wrote.
- Preferences: selected game version, addon directory URL, the unused “Check for Updates” toggle, telemetry detail level, and saved loadouts (name, game version, addon IDs).
- In-app diagnostic events, according to the telemetry detail level you select.
- Error and diagnostic messages written to local log files.

This information is processed on your device so the app can run. WinWam does not transmit it to the developer. You are not required by law to provide it. If you do not select a valid game folder, installed-addon features will not work. If the directory request cannot complete, Browse will not show a live catalog.

## Local storage

WinWam stores settings and loadouts in `%LOCALAPPDATA%\WinWam\settings.json` (or beside the executable if that location is unavailable). That file can include your WoW folder path, directory URL, and loadout names.

In-app telemetry is a session-only event history (up to 250 events) kept in memory. It is not written to disk and is discarded when WinWam exits. Despite the name, it is not sent to the developer or an analytics service. You can inspect it with **Open the Hood** in Settings.

Default telemetry detail is **Standard**. The levels are:

| Setting | What is retained in the session |
| --- | --- |
| Off | Nothing in the in-app stream. |
| Errors only | Failed operations. Messages may include addon IDs and OS error text. |
| Standard | Navigation and operations. Events may include addon IDs and loadout names. Search text is not stored; only the character count is. |
| Verbose | Reserved. It currently records the same events as Standard. |

File logging is separate from in-app telemetry. Turning telemetry **Off** does not disable log files.

When errors occur, WinWam may append to `error.txt` beside the executable. Debug builds also append diagnostic events to `log.txt`. In a release build, set `WINWAM_DEBUG_LOG=1` (or `true` / `yes`) before launch to enable `log.txt`. These files remain on your device and are not uploaded automatically. Messages may include file paths, URLs, addon IDs, or error details.

You can remove locally stored information by deleting `settings.json`, `error.txt`, and `log.txt`. Uninstalling the app may not remove files that Windows retains in local application data.

## Network requests and third parties

On launch and when you change game version or directory source, WinWam requests one public HTTPS file: `addons.<flavor>.json` from the configured directory base. That request is necessary for Browse to load the catalog.

The default base is GitHub infrastructure for the public [`bitobrian/wow-addons-directory`](https://github.com/bitobrian/wow-addons-directory) repository (`raw.githubusercontent.com`, United States). If you configure a different HTTPS addon directory, WinWam connects to that host instead. Non-HTTPS sources are rejected.

The directory host may receive standard network information such as your IP address, request time, requested URL, and HTTP client / protocol information. The developer does not receive that information from WinWam. The host’s handling of it is governed by their own privacy policy. For GitHub, see the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). The developer may also see aggregated GitHub traffic statistics for public repositories; that is not a per-user profile.

The **Check for Updates** setting is stored locally. WinWam does not currently check for application or addon updates, download addon releases, or call the GitHub API.

WinWam does not control third-party services or the privacy practices of addon sources. Production download, update, and install workflows are not implemented. If those features contact additional hosts, this policy will be updated before that behavior ships.

## Categories, purposes, sale/share, and retention

This table is the notice-at-collection style summary used by CCPA/CPRA and is also the categories/purposes/retention summary for GDPR and LGPD.

| Category | Examples | Purpose | Sold or shared? | Retention |
| --- | --- | --- | --- | --- |
| Local app data | WoW folder path, directory URL, game version, loadout names and addon IDs, telemetry level | Provide Settings, Installed, and Loadouts on your device | No. Stays on your device. | Until you change or delete `settings.json`. |
| Local diagnostics | In-app event text; `error.txt` / `log.txt` | Troubleshoot the app on your device | No. Stays on your device. | In-app events: this session, up to 250 entries, then discarded on exit. Log files: until you delete them. |
| Network metadata sent to the directory host | IP address, request time, requested `addons.<flavor>.json` URL, HTTP client information | Load the public addon catalog | No sale or sharing by WinWam. The host receives the request as a third party. | Not stored by WinWam. The host retains it under its own policy. |

WinWam does not collect sensitive personal information as that term is used under CCPA (for example government IDs, account credentials, precise geolocation, or contents of messages).

## Legal bases (GDPR and LGPD)

Where those laws apply, processing is based on:

- **Providing the app you chose to run** (GDPR Art. 6(1)(b); LGPD Art. 7, execution of a contract / requested service): local settings, folder detection, addon scanning, and the directory request that loads Browse.
- **Legitimate interests** (GDPR Art. 6(1)(f); LGPD Art. 7, legitimate interest): operating a working, local-first addon manager, including fetching a public catalog file. Those interests are to make Browse function without accounts or developer-side tracking. They are balanced against your interest in not having a user profile: the developer does not receive the request, and you can point WinWam at another HTTPS directory.

WinWam does not rely on consent for these operations, because it does not present a consent banner and does not process personal data on developer-operated servers.

## International transfers

If you use the default directory, the catalog request goes to GitHub in the United States. The United States is outside the EU/EEA, United Kingdom, and Brazil. WinWam does not itself transfer a user file or profile. The transfer, if one occurs, is the HTTPS request your device makes to the directory host.

GitHub’s processing is described in the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). If you do not want that request to reach GitHub, configure a different HTTPS addon directory that you trust.

## Your choices

- Set **Telemetry detail** to **Off** in Settings to stop the in-app event stream.
- Leave **Check for Updates** off; it has no network effect today.
- Point the addon directory at another HTTPS source, or keep the default GitHub directory.
- Delete the local files listed above.

## Your rights (GDPR, CCPA, and LGPD)

Depending on where you live, you may have rights to know or access personal information, correct it, delete it, obtain a portable copy, restrict or object to processing, withdraw consent, opt out of sale or sharing, and limit use of sensitive personal information.

Because the developer does not collect or store a user profile:

- **Access, correction, deletion, and portability of app data** are exercised on your device: change Settings, or delete `settings.json`, `error.txt`, and `log.txt`.
- **Sale / sharing / targeted advertising opt-outs** are not applicable to WinWam. The developer does not sell or share personal information and does not use it for cross-context behavioral advertising. No “Do Not Sell or Share My Personal Information” link is required for current behavior.
- **Requests to the developer:** if you believe the developer holds personal information about you, use the contact method above. If none is held, that is the response. Network logs created by a directory host must be requested from that host.
- **Complaints:** you may also lodge a complaint with a supervisory authority, such as your EU/EEA data protection authority, the UK Information Commissioner’s Office, the California Privacy Protection Agency, or Brazil’s ANPD.

These rights are not absolute. If a request is manifestly unfounded, excessive, or requires information that would identify someone else, it may be refused or limited as the applicable law allows.

## Children's privacy

WinWam is intended for people who manage a World of Warcraft installation. It is not directed to children, including children under 13 or under 16, and the developer does not knowingly collect personal information from children.

## Security

WinWam uses HTTPS (rustls) for addon directory requests and refuses non-HTTPS directory URLs. No software or transmission method can be guaranteed completely secure. Review any custom directory source before configuring it.

## Changes to this policy

This policy may be updated as WinWam changes. Updates will be published in this repository, and the effective date above will be revised when appropriate. If a change materially expands collection or network use, the update will be published before that behavior ships.

## Contact

For privacy questions or requests, open an issue in the [WinWam GitHub repository](https://github.com/bitobrian/WinWam/issues).
