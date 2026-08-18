# Data Model

## Job

```text
id
companyId
title
canonicalUrl
location
employmentType
salary
source
externalJobId
createdAt
updatedAt
```

## JobSnapshot

```text
id
jobId
capturedAt
sourceUrl
rawText
normalizedText
title
company
location
salary
description
requirements
responsibilities
extractionMetadata
```

## JobChangeSet

```text
id
jobId
fromSnapshotId
toSnapshotId
added
removed
modified
moved
reordered
addedRequirements
removedRequirements
metadata
createdAt
```

## Future objects

- Company
- JobApplication
- Document
- ResumeVersion
- CoverLetterVersion
- Snippet
- UserProfile

Do not add future entities prematurely unless a task requires them.
