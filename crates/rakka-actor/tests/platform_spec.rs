// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/{Helpers,Clock,SWARUtil}.scala and actor/src/main/java/org/apache/pekko/util/Unsafe.java (Apache-2.0).
//! UTIL-PLATAFORMA (AC-UTIL-020, 024..028): behaviours the oracle cannot pin (identity addresses, the
//! `nanoTime` source of `ScheduledClock`, bounds) and the upstream LoggerSpec assertions.

use std::cmp::Ordering;
use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rakka_actor::util::clock::{
    Clock, FixedDelayScheduler, NanoClock, ScheduledClock, SelectedClock,
};
use rakka_actor::util::fast_hash::fast_hash;
use rakka_actor::util::helpers::{
    IS_WINDOWS, compare_identity_hash, current_time_millis_to_utc_string, identity_hash_comparator,
    timestamp_at_utc_offset,
};
use rakka_actor::util::swar::{self, ByteOrder};

// ---- AC-UTIL-025: LoggerSpec.scala:283-316 ("Ticket 3080" and the padding test)

#[test]
fn utc_string_pads_every_field_with_zeros() {
    // 2019-05-05T00:00:00.000Z and 2019-05-05T23:59:59.999Z
    assert_eq!(
        current_time_millis_to_utc_string(1_557_014_400_000),
        "00:00:00.000UTC"
    );
    assert_eq!(
        current_time_millis_to_utc_string(1_557_100_799_999),
        "23:59:59.999UTC"
    );
}

#[test]
fn utc_string_equals_the_calendar_time_of_day_for_many_instants() {
    for step in 0..5000_i64 {
        let timestamp = 1_700_000_000_000 + step * 7_654_321;
        let seconds_of_day = (timestamp / 1000) % 86_400;
        let expected = format!(
            "{:02}:{:02}:{:02}.{:03}UTC",
            seconds_of_day / 3600,
            seconds_of_day / 60 % 60,
            seconds_of_day % 60,
            timestamp % 1000
        );
        assert_eq!(current_time_millis_to_utc_string(timestamp), expected);
    }
}

#[test]
fn negative_timestamp_keeps_the_sign_of_each_field() {
    assert_eq!(current_time_millis_to_utc_string(-1), "00:00:00.00-1UTC");
}

#[test]
fn timestamp_formats_month_day_year_in_the_given_offset() {
    assert_eq!(timestamp_at_utc_offset(0, 0), "01/01/1970 00:00:00.000");
    assert_eq!(
        timestamp_at_utc_offset(1_557_100_799_999, 0),
        "05/05/2019 23:59:59.999"
    );
    assert_eq!(
        timestamp_at_utc_offset(1_557_100_799_999, 3600),
        "05/06/2019 00:59:59.999"
    );
    assert_eq!(
        timestamp_at_utc_offset(0, -3 * 3600),
        "12/31/1969 21:00:00.000"
    );
}

#[test]
fn timestamp_before_the_epoch_floors_the_milliseconds() {
    assert_eq!(timestamp_at_utc_offset(-1, 0), "12/31/1969 23:59:59.999");
    assert_eq!(timestamp_at_utc_offset(-1000, 0), "12/31/1969 23:59:59.000");
}

#[test]
fn timestamp_year_is_the_year_of_era_with_a_plus_sign_past_9999() {
    assert_eq!(
        timestamp_at_utc_offset(-62_167_219_200_000, 0),
        "01/01/0001 00:00:00.000"
    ); // year 0
    assert_eq!(
        timestamp_at_utc_offset(253_402_300_800_000, 0),
        "01/01/+10000 00:00:00.000"
    );
}

// ---- AC-UTIL-026

#[test]
fn is_windows_follows_the_target_platform() {
    assert_eq!(IS_WINDOWS, cfg!(target_os = "windows"));
}

// ---- AC-UTIL-024: identity order (no oracle: identityHashCode is not reproducible)

#[test]
fn identity_order_is_consistent_with_the_address() {
    let items = [1_u32, 2, 3];
    let (first, last) = (&items[0], &items[2]);
    assert_eq!(compare_identity_hash(first, first), Ordering::Equal);
    assert_eq!(compare_identity_hash(first, last), Ordering::Less);
    assert_eq!(compare_identity_hash(last, first), Ordering::Greater);
}

#[test]
fn identity_order_works_for_unsized_referents() {
    let text = String::from("same text");
    let a: &str = &text;
    assert_eq!(compare_identity_hash(a, a), Ordering::Equal);
}

#[test]
fn comparator_falls_back_when_identities_tie_for_unequal_objects() {
    // NaN is not equal to itself, so the same object ties by identity and differs by `==`: Pekko's
    // `case 0 if a != b` branch.
    let nan = f64::NAN;
    let calls = Mutex::new(0);
    let comparator = identity_hash_comparator(|_: &f64, _: &f64| {
        *calls.lock().unwrap() += 1;
        Ordering::Greater
    });
    assert_eq!(comparator(&nan, &nan), Ordering::Greater);
    assert_eq!(*calls.lock().unwrap(), 1);
    let one = 1.0_f64;
    assert_eq!(comparator(&one, &one), Ordering::Equal);
    assert_eq!(
        *calls.lock().unwrap(),
        1,
        "equal objects never reach the fallback"
    );
}

// ---- AC-UTIL-027: ScheduledClock with a time source driven by hand

#[derive(Default)]
struct RecordingScheduler {
    delays: Mutex<Vec<(Duration, Duration)>>,
    tasks: Mutex<Vec<Box<dyn Fn() + Send + Sync>>>,
}

impl FixedDelayScheduler for RecordingScheduler {
    fn schedule_with_fixed_delay(
        &self,
        initial_delay: Duration,
        delay: Duration,
        task: Box<dyn Fn() + Send + Sync>,
    ) {
        self.delays.lock().unwrap().push((initial_delay, delay));
        self.tasks.lock().unwrap().push(task);
    }
}

fn clock_with_manual_time(
    interval: Duration,
    start: i64,
) -> (Arc<ScheduledClock>, Arc<AtomicI64>, RecordingScheduler) {
    let now = Arc::new(AtomicI64::new(start));
    let scheduler = RecordingScheduler::default();
    let source = Arc::clone(&now);
    let clock = ScheduledClock::with_time_source(
        interval,
        &scheduler,
        Box::new(move || source.load(AtomicOrdering::SeqCst)),
    );
    (clock, now, scheduler)
}

#[test]
fn scheduled_clock_schedules_its_update_with_the_interval_as_initial_delay_and_delay() {
    let (_clock, _now, scheduler) = clock_with_manual_time(Duration::from_millis(7), 0);
    assert_eq!(
        *scheduler.delays.lock().unwrap(),
        vec![(Duration::from_millis(7), Duration::from_millis(7))]
    );
}

#[test]
fn scheduled_clock_starts_at_the_time_source_and_increments_by_one() {
    let (clock, _now, _scheduler) = clock_with_manual_time(Duration::from_millis(1), 1000);
    assert_eq!(clock.earlier_time(0), 1000);
    assert_eq!(clock.current_time(), 1001);
    assert_eq!(clock.current_time(), 1002);
    assert_eq!(clock.earlier_time(2), 1000);
    assert_eq!(clock.earlier_time(-5), 1007);
}

#[test]
fn scheduled_clock_update_follows_the_time_source_forward() {
    let (clock, now, scheduler) = clock_with_manual_time(Duration::from_millis(1), 1000);
    clock.current_time();
    now.store(5000, AtomicOrdering::SeqCst);
    (scheduler.tasks.lock().unwrap()[0])();
    assert_eq!(clock.earlier_time(0), 5000);
    assert_eq!(clock.current_time(), 5001);
}

#[test]
fn scheduled_clock_never_moves_backwards() {
    // The increments ran ahead of the time source: an update keeps the larger value.
    let (clock, now, _scheduler) = clock_with_manual_time(Duration::from_millis(1), 1000);
    for _ in 0..50 {
        clock.current_time();
    }
    now.store(1010, AtomicOrdering::SeqCst);
    clock.update();
    assert_eq!(clock.earlier_time(0), 1050);
}

#[test]
fn scheduled_clock_update_handles_the_overflow_to_negative() {
    // Like `nanoTime`, the value wraps past Long.MaxValue: the difference test still sees it as newer.
    let (clock, now, _scheduler) = clock_with_manual_time(Duration::from_millis(1), i64::MAX - 5);
    now.store(i64::MIN + 10, AtomicOrdering::SeqCst);
    clock.update();
    assert_eq!(clock.earlier_time(0), i64::MIN + 10);
    assert_eq!(clock.current_time(), i64::MIN + 11);
}

#[test]
fn scheduled_clock_stops_incrementing_after_the_budget_until_the_next_update() {
    // interval 100.002 us: budget of two increments
    let (clock, now, _scheduler) = clock_with_manual_time(Duration::from_nanos(100_002), 0);
    assert_eq!(clock.current_time(), 1);
    assert!(!clock.max_increment_reached());
    assert_eq!(clock.current_time(), 2);
    assert!(clock.max_increment_reached());
    assert_eq!(clock.current_time(), 2);
    now.store(10, AtomicOrdering::SeqCst);
    clock.update();
    assert!(!clock.max_increment_reached());
    assert_eq!(clock.current_time(), 11);
}

#[test]
fn scheduled_clock_with_a_short_interval_has_no_budget() {
    let (clock, _now, _scheduler) = clock_with_manual_time(Duration::from_nanos(1), 0);
    assert_eq!(clock.current_time(), 1);
    assert!(clock.max_increment_reached());
    assert_eq!(clock.current_time(), 1);
}

#[test]
fn scheduled_clock_hands_out_distinct_values_to_concurrent_callers() {
    let (clock, _now, _scheduler) = clock_with_manual_time(Duration::from_secs(1000), 0);
    let mut handles = Vec::new();
    for _ in 0..4 {
        let clock = Arc::clone(&clock);
        handles.push(std::thread::spawn(move || {
            (0..1000).map(|_| clock.current_time()).collect::<Vec<_>>()
        }));
    }
    let mut seen: Vec<i64> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 4000);
}

#[test]
fn selection_uses_the_scheduled_clock_only_for_a_positive_interval() {
    let scheduler = RecordingScheduler::default();
    let nano = SelectedClock::for_interval(Duration::ZERO, &scheduler);
    assert_eq!(nano.class_name(), "NanoClock");
    assert!(scheduler.tasks.lock().unwrap().is_empty());
    let scheduled = SelectedClock::for_interval(Duration::from_secs(1), &scheduler);
    assert_eq!(scheduled.class_name(), "ScheduledClock");
    assert_eq!(scheduler.tasks.lock().unwrap().len(), 1);
}

#[test]
fn selected_clock_delegates_to_the_chosen_clock() {
    let scheduler = RecordingScheduler::default();
    let nano = SelectedClock::for_interval(Duration::ZERO, &scheduler);
    let before = nano.current_time();
    assert!(nano.earlier_time(0).wrapping_sub(before) >= 0);
    let scheduled = SelectedClock::for_interval(Duration::from_secs(1), &scheduler);
    let base = scheduled.earlier_time(0);
    assert_eq!(scheduled.current_time(), base + 1);
}

#[test]
fn nano_clock_earlier_time_is_the_current_time_minus_the_duration() {
    let clock = NanoClock;
    let (before, earlier, after) = (
        clock.current_time(),
        clock.earlier_time(1_000),
        clock.current_time(),
    );
    assert!(earlier.wrapping_add(1_000).wrapping_sub(before) >= 0);
    assert!(after.wrapping_sub(earlier.wrapping_add(1_000)) >= 0);
}

// ---- AC-UTIL-028: SWARUtil

#[test]
fn applied_pattern_flags_exactly_the_matching_bytes() {
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..20_000 {
        let probe = [0x00, 0x7F, 0x80, 0xFE, 0xFF][(next() % 5) as usize];
        // Plant the probe in the lanes selected by a random mask: bytes beside the carry boundaries too.
        let (mut word, planted_lanes) = (next(), next());
        for lane in 0..8 {
            if planted_lanes >> lane & 1 == 1 {
                word = word & !(0xFF << (8 * lane)) | u64::from(probe) << (8 * lane);
            }
        }
        let applied = swar::apply_pattern(word as i64, swar::compile_pattern(probe)) as u64;
        for lane in 0..8 {
            let matches = ((word >> (8 * lane)) & 0xFF) as u8 == probe;
            assert_eq!(applied >> (8 * lane) & 0xFF, if matches { 0x80 } else { 0 });
        }
    }
}

#[test]
fn first_and_last_index_locate_matches_from_the_most_significant_byte() {
    let word = swar::apply_pattern(0x00_11_22_11_33_44_11_55, swar::compile_pattern(0x11));
    assert_eq!(swar::get_index(word), 1);
    assert_eq!(swar::get_last_index(word), 6);
    assert_eq!(swar::get_index(0), 8);
    assert_eq!(swar::get_last_index(0), -1);
}

#[test]
fn what_is_put_is_read_back_in_both_orders() {
    for order in [ByteOrder::BigEndian, ByteOrder::LittleEndian] {
        let mut array = [0_u8; 16];
        swar::put_long(&mut array, 3, -0x0123_4567_89AB_CDEF, order);
        assert_eq!(swar::get_long(&array, 3, order), -0x0123_4567_89AB_CDEF);
        swar::put_int(&mut array, 1, i32::MIN + 7, order);
        assert_eq!(swar::get_int(&array, 1, order), i32::MIN + 7);
        swar::put_short(&mut array, 9, 0x0001_8001, order);
        assert_eq!(
            swar::get_short(&array, 9, order),
            i16::MIN + 1,
            "only the low 16 bits are written"
        );
    }
}

#[test]
fn big_and_little_endian_read_the_same_bytes_in_opposite_order() {
    let array = [1_u8, 2, 3, 4];
    assert_eq!(swar::get_int(&array, 0, ByteOrder::BigEndian), 0x0102_0304);
    assert_eq!(
        swar::get_int(&array, 0, ByteOrder::LittleEndian),
        0x0403_0201
    );
}

#[test]
#[should_panic]
fn reading_past_the_end_panics_like_pekkos_index_out_of_bounds() {
    let _ = swar::get_long(&[0_u8; 8], 1, ByteOrder::BigEndian);
}

// ---- AC-UTIL-020: Unsafe.fastHash

#[test]
fn fast_hash_matches_the_measured_values_for_ascii() {
    assert_eq!(fast_hash(""), Some(992_666));
    assert_eq!(fast_hash("abc"), Some(1_177_643_766));
}

#[test]
fn fast_hash_of_non_ascii_text_waits_for_the_owner() {
    // Q-UTIL-19 (proposed): Pekko's answer depends on the JVM's algorithm.
    assert_eq!(fast_hash("\u{e9}"), None);
    assert_eq!(fast_hash("a\u{1F600}"), None);
}
