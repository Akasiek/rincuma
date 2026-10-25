#![allow(clippy::panic_in_result_fn)]

use super::{End, Frequency, Schedule, ScheduleError};
use jiff::{Timestamp, civil::date};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn schedule(
    anchor: &str,
    frequency: Frequency,
    interval: u32,
    zone: &str,
    end: End,
) -> Result<Schedule, Box<dyn std::error::Error>> {
    Ok(Schedule::new(
        frequency,
        interval,
        zone,
        anchor.parse()?,
        end,
    )?)
}

fn assert_next(
    schedule: &Schedule,
    original: &str,
    completion: &str,
    count: u32,
    expected: Option<&str>,
) -> TestResult {
    assert_eq!(
        schedule.next_after(original.parse()?, completion.parse()?, count)?,
        expected.map(str::parse::<Timestamp>).transpose()?,
    );
    Ok(())
}

#[test]
fn daily_intervals_skip_elapsed_slots() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        2,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-15T12:00:00Z",
        1,
        Some("2026-10-16T09:00:00Z"),
    )
}

#[test]
fn early_completion_advances_beyond_the_original_slot() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-11T09:00:00Z",
        "2026-10-10T12:00:00Z",
        2,
        Some("2026-10-12T09:00:00Z"),
    )
}

#[test]
fn completion_exactly_on_a_slot_skips_that_slot() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        2,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-14T09:00:00Z",
        1,
        Some("2026-10-16T09:00:00Z"),
    )
}

#[test]
fn completion_just_before_a_slot_keeps_that_slot() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        2,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-14T08:59:59.999999999Z",
        1,
        Some("2026-10-14T09:00:00Z"),
    )
}

#[test]
fn weekly_intervals_keep_the_anchor_weekday_across_years() -> TestResult {
    let schedule = schedule(
        "2026-12-25T09:00:00",
        Frequency::Weekly,
        2,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-12-25T09:00:00Z",
        "2027-01-10T12:00:00Z",
        1,
        Some("2027-01-22T09:00:00Z"),
    )
}

#[test]
fn month_clamping_does_not_drift_after_february() -> TestResult {
    for (year, last_day) in [(2026, 28), (2028, 29)] {
        let schedule = schedule(
            &format!("{year}-01-31T09:00:00"),
            Frequency::Monthly,
            1,
            "UTC",
            End::Never,
        )?;
        let january = format!("{year}-01-31T09:00:00Z");
        let february = format!("{year}-02-{last_day}T09:00:00Z");
        let march = format!("{year}-03-31T09:00:00Z");
        assert_next(&schedule, &january, &january, 1, Some(&february))?;
        assert_next(&schedule, &february, &february, 2, Some(&march))?;
    }
    Ok(())
}

#[test]
fn monthly_intervals_cross_year_boundaries() -> TestResult {
    let schedule = schedule(
        "2026-11-30T09:00:00",
        Frequency::Monthly,
        3,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-11-30T09:00:00Z",
        "2026-11-30T09:00:00Z",
        1,
        Some("2027-02-28T09:00:00Z"),
    )?;
    assert_next(
        &schedule,
        "2027-02-28T09:00:00Z",
        "2027-02-28T09:00:00Z",
        2,
        Some("2027-05-30T09:00:00Z"),
    )
}

#[test]
fn daily_slots_preserve_wall_time_across_both_dst_transitions() -> TestResult {
    for (anchor, original, expected) in [
        (
            "2026-03-28T09:00:00",
            "2026-03-28T08:00:00Z",
            "2026-03-29T07:00:00Z",
        ),
        (
            "2026-10-24T09:00:00",
            "2026-10-24T07:00:00Z",
            "2026-10-25T08:00:00Z",
        ),
    ] {
        let schedule = schedule(anchor, Frequency::Daily, 1, "Europe/Warsaw", End::Never)?;
        assert_next(&schedule, original, original, 1, Some(expected))?;
    }
    Ok(())
}

#[test]
fn spring_gap_moves_forward_without_changing_future_wall_time() -> TestResult {
    let schedule = schedule(
        "2026-03-28T02:30:00",
        Frequency::Daily,
        1,
        "Europe/Warsaw",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-03-28T01:30:00Z",
        "2026-03-28T01:30:00Z",
        1,
        Some("2026-03-29T01:30:00Z"),
    )?;
    assert_next(
        &schedule,
        "2026-03-29T01:30:00Z",
        "2026-03-29T01:30:00Z",
        2,
        Some("2026-03-30T00:30:00Z"),
    )
}

#[test]
fn autumn_fold_uses_the_earlier_instant() -> TestResult {
    let schedule = schedule(
        "2026-10-24T02:30:00",
        Frequency::Daily,
        1,
        "Europe/Warsaw",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-24T00:30:00Z",
        "2026-10-24T00:30:00Z",
        1,
        Some("2026-10-25T00:30:00Z"),
    )?;
    // Completing between the two readings must not produce a second occurrence.
    assert_next(
        &schedule,
        "2026-10-24T00:30:00Z",
        "2026-10-25T01:00:00Z",
        1,
        Some("2026-10-26T01:30:00Z"),
    )
}

#[test]
fn half_hour_dst_gap_uses_the_actual_transition_gap() -> TestResult {
    let schedule = schedule(
        "2026-10-03T02:15:00",
        Frequency::Daily,
        1,
        "Australia/Lord_Howe",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-02T15:45:00Z",
        "2026-10-02T15:45:00Z",
        1,
        Some("2026-10-03T15:45:00Z"),
    )
}

#[test]
fn an_anchor_in_a_gap_retains_its_intended_wall_time() -> TestResult {
    let schedule = schedule(
        "2026-03-29T02:30:00",
        Frequency::Weekly,
        1,
        "Europe/Warsaw",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-03-29T01:30:00Z",
        "2026-03-29T01:30:00Z",
        1,
        Some("2026-04-05T00:30:00Z"),
    )
}

#[test]
fn skipped_calendar_date_is_checked_before_the_estimated_slot() -> TestResult {
    let schedule = schedule(
        "2011-12-29T09:00:00",
        Frequency::Daily,
        1,
        "Pacific/Apia",
        End::Through(date(2011, 12, 30)),
    )?;
    // December 30 was skipped. Its 09:00 slot resolves to December 31 at 09:00.
    // It still belongs to the inclusive December 30 schedule end date.
    assert_next(
        &schedule,
        "2011-12-29T19:00:00Z",
        "2011-12-30T18:00:00Z",
        1,
        Some("2011-12-30T19:00:00Z"),
    )
}

#[test]
fn end_date_is_inclusive_in_the_series_time_zone() -> TestResult {
    let schedule = schedule(
        "2026-10-24T23:30:00",
        Frequency::Daily,
        1,
        "America/New_York",
        End::Through(date(2026, 10, 25)),
    )?;
    assert_next(
        &schedule,
        "2026-10-25T03:30:00Z",
        "2026-10-25T03:30:00Z",
        1,
        Some("2026-10-26T03:30:00Z"),
    )?;
    assert_next(
        &schedule,
        "2026-10-26T03:30:00Z",
        "2026-10-26T03:30:00Z",
        2,
        None,
    )
}

#[test]
fn late_completion_after_the_end_date_creates_no_successor() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::Through(date(2026, 10, 12)),
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-15T09:00:00Z",
        1,
        None,
    )
}

#[test]
fn occurrence_limit_counts_created_tasks_instead_of_skipped_slots() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::After(2),
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-11-01T12:00:00Z",
        1,
        Some("2026-11-02T09:00:00Z"),
    )?;
    assert_next(
        &schedule,
        "2026-11-02T09:00:00Z",
        "2026-11-02T09:00:00Z",
        2,
        None,
    )
}

#[test]
fn a_limit_of_one_makes_the_first_task_the_last() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::After(1),
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-10T09:00:00Z",
        1,
        None,
    )
}

#[test]
fn restoring_a_deleted_successors_count_allows_recalculation() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::After(2),
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-11T12:00:00Z",
        2,
        None,
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-11T12:00:00Z",
        1,
        Some("2026-10-12T09:00:00Z"),
    )
}

#[test]
fn a_changed_rule_uses_its_new_anchor() -> TestResult {
    let schedule = schedule(
        "2026-10-15T14:00:00",
        Frequency::Weekly,
        1,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "2026-10-10T09:00:00Z",
        "2026-10-14T12:00:00Z",
        3,
        Some("2026-10-22T14:00:00Z"),
    )
}

#[test]
fn long_inactive_series_skips_directly_to_a_recent_slot() -> TestResult {
    let schedule = schedule(
        "1900-01-01T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::Never,
    )?;
    assert_next(
        &schedule,
        "1900-01-01T09:00:00Z",
        "9000-10-25T12:00:00Z",
        1,
        Some("9000-10-26T09:00:00Z"),
    )
}

#[test]
fn invalid_configuration_is_rejected() -> TestResult {
    let anchor = "2026-10-10T09:00:00".parse()?;
    assert!(matches!(
        Schedule::new(Frequency::Daily, 0, "UTC", anchor, End::Never),
        Err(ScheduleError::InvalidInterval),
    ));
    assert!(matches!(
        Schedule::new(Frequency::Daily, 1, "UTC", anchor, End::After(0)),
        Err(ScheduleError::InvalidLimit),
    ));
    for zone in [
        "Not/A_Zone",
        "+02:00",
        "",
        "EST5EDT,M3.2.0,M11.1.0",
        "Etc/Unknown",
    ] {
        assert!(matches!(
            Schedule::new(Frequency::Daily, 1, zone, anchor, End::Never),
            Err(ScheduleError::InvalidTimeZone),
        ));
    }
    Ok(())
}

#[test]
fn occurrence_count_must_be_valid_and_fit_the_limit() -> TestResult {
    let schedule = schedule(
        "2026-10-10T09:00:00",
        Frequency::Daily,
        1,
        "UTC",
        End::After(2),
    )?;
    let now = "2026-10-10T09:00:00Z".parse()?;
    assert!(matches!(
        schedule.next_after(now, now, 0),
        Err(ScheduleError::InvalidCount),
    ));
    assert!(matches!(
        schedule.next_after(now, now, 3),
        Err(ScheduleError::LimitBelowCount),
    ));
    Ok(())
}

#[test]
fn date_overflow_returns_an_error_instead_of_panicking() -> TestResult {
    let schedule = schedule(
        "9999-12-01T09:00:00",
        Frequency::Monthly,
        1,
        "UTC",
        End::Never,
    )?;
    let now = "9999-12-01T09:00:00Z".parse()?;
    assert!(matches!(
        schedule.next_after(now, now, 1),
        Err(ScheduleError::Calendar(_)),
    ));
    Ok(())
}

#[test]
fn oversized_intervals_return_an_error_instead_of_panicking() -> TestResult {
    for frequency in [Frequency::Daily, Frequency::Weekly, Frequency::Monthly] {
        let schedule = schedule(
            "2026-10-10T09:00:00",
            frequency,
            u32::MAX,
            "UTC",
            End::Never,
        )?;
        let now = "2026-10-10T09:00:00Z".parse()?;
        assert!(matches!(
            schedule.next_after(now, now, 1),
            Err(ScheduleError::Calendar(_)),
        ));
    }
    Ok(())
}
