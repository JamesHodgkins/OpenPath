# Corpus manifest schema (P0-01)

Every fixture ships with a sidecar manifest so a check-in can be reproduced and its
fidelity expectation is machine-readable. Proposed schema, to be finalized during P0-01.

```yaml
# Example: corpus/mpp/acme-2024.mpp.manifest.yaml
format: mpp14                 # mpp14 | mspdi | native
source: synthetic             # synthetic | customer | public
permission_log_id: PL-001     # references permission-log.csv
checksum_sha256: "<hex>"
producer:
  tool: "Microsoft Project"
  version: "365"              # e.g. 2016, 2019, 365, build number
  build: "16.0.17xxx.xxxxx"
project_options:              # §5.2 — every option that affects scheduling
  schedule_from: start
  minutes_per_day: 480
  minutes_per_week: 2400
  days_per_month: 20
  default_start_time: "08:00"
  default_finish_time: "17:00"
  week_start_day: monday
  critical_slack_limit: 0
  calculate_multiple_critical_paths: false
  scheduling_ignores_resource_calendars: false
  default_task_type: fixed_units
  default_effort_driven: true
expected:                     # §9.2 compared fields
  fields: [start, finish, duration, work, cost, total_slack, free_slack, critical, actual_start, actual_finish]
  tolerance:
    dates: minute
    duration: tick
    money: "0.01"
  fidelity_baseline:          # expected counts by class, checked by CI
    full: 15
    read_only: 0
    lossy: 1
    computed: 0
    ignored: 8
```

## Notes

- `checksum_sha256` is over the fixture bytes, computed by `tools/coverage`.
- The manifest is the ground truth for the differential harness; cached MSPDI values
  in the fixture are never trusted as inputs (§9.2).
- Regenerating the parity corpus is a reviewed commit, not an incidental change.
