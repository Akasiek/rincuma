#![allow(clippy::panic_in_result_fn)]

use super::{End, Frequency, Pattern, Schedule};
use jiff::{Timestamp, civil::date};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn assert_slots(
    frequency: Frequency,
    interval: u32,
    anchor: &str,
    pattern: Pattern,
    cases: &[(&str, &str, &str)],
) -> TestResult {
    let schedule = Schedule::new(
        frequency,
        interval,
        "Europe/Warsaw",
        anchor.parse()?,
        End::Never,
    )?
    .with_pattern(Some(pattern))?;
    for (original, completion, expected) in cases {
        assert_eq!(
            schedule.next_after(original.parse()?, completion.parse()?, 1)?,
            Some(expected.parse::<Timestamp>()?),
            "{anchor}, {completion}"
        );
    }
    Ok(())
}

#[test]
fn selected_weekdays_advance_within_the_anchor_week() -> TestResult {
    assert_slots(
        Frequency::Weekly,
        1,
        "2026-10-05T09:00:00",
        Pattern::Weekdays { days: vec![1, 4] },
        &[
            (
                "2026-10-05T07:00:00Z",
                "2026-10-05T07:05:00Z",
                "2026-10-08T07:00:00Z",
            ),
            (
                "2026-10-08T07:00:00Z",
                "2026-10-08T07:05:00Z",
                "2026-10-12T07:00:00Z",
            ),
        ],
    )
}

#[test]
fn selected_weekdays_anchor_interval_weeks_to_monday_across_years() -> TestResult {
    assert_slots(
        Frequency::Weekly,
        2,
        "2026-12-30T09:00:00",
        Pattern::Weekdays { days: vec![1, 4] },
        &[
            (
                "2026-12-31T08:00:00Z",
                "2026-12-31T08:05:00Z",
                "2027-01-11T08:00:00Z",
            ),
            (
                "2026-12-31T08:00:00Z",
                "2027-01-12T10:00:00Z",
                "2027-01-14T08:00:00Z",
            ),
        ],
    )
}

#[test]
fn selected_weekdays_handle_early_completion_and_dst() -> TestResult {
    assert_slots(
        Frequency::Weekly,
        1,
        "2026-10-18T09:00:00",
        Pattern::Weekdays { days: vec![1, 7] },
        &[(
            "2026-10-19T07:00:00Z",
            "2026-10-18T08:00:00Z",
            "2026-10-25T08:00:00Z",
        )],
    )
}

#[test]
fn monthly_second_friday_handles_month_and_year_intervals() -> TestResult {
    assert_slots(
        Frequency::Monthly,
        2,
        "2026-12-01T09:00:00",
        Pattern::MonthlyWeekday {
            ordinal: 2,
            weekday: 5,
        },
        &[
            (
                "2026-12-01T08:00:00Z",
                "2026-12-01T10:00:00Z",
                "2026-12-11T08:00:00Z",
            ),
            (
                "2026-12-11T08:00:00Z",
                "2026-12-11T10:00:00Z",
                "2027-02-12T08:00:00Z",
            ),
        ],
    )
}

#[test]
fn monthly_fifth_friday_skips_months_without_changing_intent() -> TestResult {
    assert_slots(
        Frequency::Monthly,
        1,
        "2027-01-29T09:00:00",
        Pattern::MonthlyWeekday {
            ordinal: 5,
            weekday: 5,
        },
        &[(
            "2027-01-29T08:00:00Z",
            "2027-01-29T10:00:00Z",
            "2027-04-30T07:00:00Z",
        )],
    )
}

#[test]
fn monthly_last_friday_handles_leap_years_and_dst() -> TestResult {
    assert_slots(
        Frequency::Monthly,
        1,
        "2028-02-25T09:00:00",
        Pattern::MonthlyWeekday {
            ordinal: -1,
            weekday: 5,
        },
        &[(
            "2028-02-25T08:00:00Z",
            "2028-02-25T10:00:00Z",
            "2028-03-31T07:00:00Z",
        )],
    )
}

#[test]
fn advanced_patterns_preserve_end_modes_and_created_counts() -> TestResult {
    for pattern in [
        Pattern::Weekdays { days: vec![1, 4] },
        Pattern::MonthlyWeekday {
            ordinal: 5,
            weekday: 5,
        },
    ] {
        let frequency = if matches!(pattern, Pattern::Weekdays { .. }) {
            Frequency::Weekly
        } else {
            Frequency::Monthly
        };
        let scheduled: Timestamp = "2027-01-29T09:00:00Z".parse()?;
        let limited = Schedule::new(
            frequency,
            1,
            "UTC",
            "2027-01-29T09:00:00".parse()?,
            End::After(2),
        )?
        .with_pattern(Some(pattern.clone()))?;
        assert!(limited.next_after(scheduled, scheduled, 1)?.is_some());
        assert_eq!(limited.next_after(scheduled, scheduled, 2)?, None);
        let ended = Schedule::new(
            frequency,
            1,
            "UTC",
            "2027-01-29T09:00:00".parse()?,
            End::Through(date(2027, 1, 29)),
        )?
        .with_pattern(Some(pattern))?;
        assert_eq!(ended.next_after(scheduled, scheduled, 1)?, None);
    }
    Ok(())
}

#[test]
fn invalid_or_incompatible_patterns_are_rejected() -> TestResult {
    let invalid = [
        (Frequency::Weekly, Pattern::Weekdays { days: vec![] }),
        (Frequency::Weekly, Pattern::Weekdays { days: vec![1, 1] }),
        (Frequency::Weekly, Pattern::Weekdays { days: vec![0] }),
        (Frequency::Weekly, Pattern::Weekdays { days: vec![8] }),
        (Frequency::Daily, Pattern::Weekdays { days: vec![1] }),
        (
            Frequency::Weekly,
            Pattern::MonthlyWeekday {
                ordinal: 1,
                weekday: 5,
            },
        ),
        (
            Frequency::Monthly,
            Pattern::MonthlyWeekday {
                ordinal: 0,
                weekday: 5,
            },
        ),
        (
            Frequency::Monthly,
            Pattern::MonthlyWeekday {
                ordinal: 6,
                weekday: 5,
            },
        ),
        (
            Frequency::Monthly,
            Pattern::MonthlyWeekday {
                ordinal: -2,
                weekday: 5,
            },
        ),
        (
            Frequency::Monthly,
            Pattern::MonthlyWeekday {
                ordinal: 1,
                weekday: 8,
            },
        ),
    ];
    for (frequency, pattern) in invalid {
        assert!(
            Schedule::new(
                frequency,
                1,
                "UTC",
                "2026-10-05T09:00:00".parse()?,
                End::Never
            )?
            .with_pattern(Some(pattern))
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn monthly_patterns_preserve_dst_gap_and_fold_intent() -> TestResult {
    let schedule = Schedule::new(
        Frequency::Monthly,
        1,
        "Europe/Warsaw",
        "2026-02-22T02:30:00".parse()?,
        End::Never,
    )?
    .with_pattern(Some(Pattern::MonthlyWeekday {
        ordinal: -1,
        weekday: 7,
    }))?;
    for (original, completion, expected) in [
        (
            "2026-02-22T01:30:00Z",
            "2026-02-22T01:35:00Z",
            "2026-03-29T01:30:00Z",
        ),
        (
            "2026-03-29T01:30:00Z",
            "2026-03-29T01:35:00Z",
            "2026-04-26T00:30:00Z",
        ),
        (
            "2026-09-27T00:30:00Z",
            "2026-09-27T00:35:00Z",
            "2026-10-25T00:30:00Z",
        ),
    ] {
        assert_eq!(
            schedule.next_after(original.parse()?, completion.parse()?, 1)?,
            Some(expected.parse()?)
        );
    }
    Ok(())
}

#[test]
fn impossible_fifth_weekday_interval_finishes_without_scanning_unboundedly() -> TestResult {
    let schedule = Schedule::new(
        Frequency::Monthly,
        4800,
        "UTC",
        "2027-02-01T09:00:00".parse()?,
        End::Never,
    )?
    .with_pattern(Some(Pattern::MonthlyWeekday {
        ordinal: 5,
        weekday: 5,
    }))?;
    let original = "2027-02-01T09:00:00Z".parse()?;
    assert_eq!(schedule.next_after(original, original, 1)?, None);
    Ok(())
}
