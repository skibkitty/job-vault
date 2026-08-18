# Privacy Specification

## Privacy promise

The application is local-first.

The extension and companion must not transmit saved job-search information to remote services unless a future feature is explicitly approved by the user.

## Explicit collection model

The extension should collect job information when the user invokes a relevant feature such as:

> Save Job

It should not continuously monitor arbitrary browsing.

## Allowed job data

Only information relevant to job searching, such as:

- title;
- company;
- location;
- salary;
- job description;
- requirements;
- responsibilities;
- job URL;
- source;
- timestamps;
- application state;
- user notes;
- resume/cover-letter data if a future feature adds them.

## Forbidden unrelated data

Do not collect:

- browser history;
- passwords;
- cookies;
- tokens;
- payment data;
- arbitrary websites;
- unrelated page content;
- advertising data;
- clipboard history.

## Privacy regression rule

Any feature that could expand data collection must explicitly document:

- what data is collected;
- why;
- when;
- where it is stored;
- whether it leaves the machine;
- how it is deleted.

## Future AI

Remote AI is prohibited by default.

A future local AI provider is acceptable if it operates entirely on the user's machine.

A remote AI provider would require explicit user approval and a change to the privacy model.
