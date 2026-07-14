# Schedules

Komodo can run [**Procedures** and **Actions**](procedures) automatically on a configured schedule.

## Configuration

Add scheduling fields to any Procedure or Action:

```toml
[[procedure]]
name = "nightly-backup"
[procedure.config]
schedule_format = "English"
schedule = "Every day at 03:00"
schedule_enabled = true
schedule_timezone = "America/New_York"
schedule_alert = true
failure_alert = true
```

### Schedule fields

| Field | Description | Default |
| --- | --- | --- |
| `schedule_format` | `English` for natural language, or `Cron` for cron expressions. | `English` |
| `schedule` | The schedule expression (see formats below). | `""` |
| `schedule_enabled` | Whether the schedule is active. | `true` |
| `schedule_timezone` | [TZ identifier](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) (e.g. `America/New_York`). Uses Core's timezone if empty. | `""` |
| `schedule_alert` | Send an alert each time the schedule runs. | `true` |
| `failure_alert` | Send an alert when a scheduled run fails. | `true` |

## Schedule formats

### English (natural language)

Set `schedule_format = "English"` and write the schedule as a sentence:

- `Every day at 03:00`
- `Every 5 minutes`
- `At midnight on the 1st and 15th of the month`
- `Every Monday at 09:00`

Komodo converts these to cron expressions internally using the
[english-to-cron](https://crates.io/crates/english-to-cron) crate.

### Cron

Set `schedule_format = "Cron"` and provide a 6-field cron expression (**seconds are required**):

```text
second  minute  hour  day  month  day-of-week
```

Examples:

| Expression | Meaning |
| --- | --- |
| `0 0 3 * * ?` | Every day at 03:00:00 |
| `0 */5 * * * ?` | Every 5 minutes |
| `0 0 0 1,15 * ?` | At midnight on the 1st and 15th |
| `0 0 9 ? * MON` | Every Monday at 09:00 |

## Viewing schedules

The **ListSchedules** API endpoint returns all configured schedules with their status, including:

- Last run time
- Next scheduled run time
- Any schedule parse errors

## Alerts

When `schedule_alert` is enabled, Komodo sends an alert through your configured
[Alerters](../alerter.md) each time a scheduled Procedure or Action runs. If `failure_alert` is
enabled, an additional alert is sent when the run fails.
