# Permission Policy

## Principle

Use the minimum Chrome permissions necessary. Every permission must have a documented reason.

## Required permissions

### `nativeMessaging`

Reason: Communication with the local Rust companion via Chrome Native Messaging.

Required from initial implementation. Without this, the extension cannot send data to or receive data from the companion that owns the encrypted database.

## Recommended permissions

### `activeTab`

Reason: Access the current tab's content when the user explicitly invokes a save action (e.g., clicking "Save Job").

This avoids requesting host permissions for every site. `activeTab` grants temporary access to the active tab only after a user gesture (button click, context menu).

### `scripting`

Reason: Inject content scripts into the active tab to extract job information when the user invokes a save action.

Used alongside `activeTab` to run extraction on demand.

## Permissions not requested

### `<all_urls>` host permission

Rejected. The extension does not need to run on every page. It processes job pages only when the user explicitly invokes a save action, and `activeTab` + `scripting` provide that access on demand.

### `tabs`

Rejected. The extension does not need to enumerate or monitor all open tabs.

### `storage`

Rejected for now. The extension does not persist sensitive data locally. All sensitive storage goes through the companion. If local non-sensitive caching is later needed, this permission will be re-evaluated.

### `cookies`

Rejected. The extension does not need access to cookies.

### `webRequest` / `webRequestBlocking`

Rejected. The extension does not intercept or modify network requests.

### `downloads`

Rejected. The extension does not trigger downloads.

### `notifications`

Rejected for now. May be reconsidered if background alerts are needed.

## Site access strategy

The extension should not request blanket host permissions for LinkedIn, Indeed, or other job sites.

Approach:

1. User clicks the extension action button or uses a keyboard shortcut.
2. Extension uses `activeTab` to access the current tab.
3. Extension uses `scripting` to inject the appropriate adapter.
4. Adapter extracts job information from the page DOM.

If a future feature requires background processing of specific sites (e.g., monitoring for reposts), host permissions for those specific origins may be requested with:

- documented justification;
- ADR;
- human review;
- update to this document.

## Optional permissions

Future features may use Chrome's optional permissions model to request additional access only when the user enables a specific feature. For example:

- `storage` if local caching is added;
- specific host permissions if background monitoring is added.

Optional permissions must be:

- requested only when the feature is enabled;
- documented in this file;
- approved by the user through Chrome's permission prompt.

## Audit

Before each release, review:

1. Every permission in `manifest.json` has a matching entry in this document.
2. No permission is present without justification.
3. No `<all_urls>` or broad host permission exists without an ADR.
4. No new permission has been added since the last review.
