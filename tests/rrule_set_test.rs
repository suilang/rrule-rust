use chrono_tz::Tz;
use rrule_rust::{point_time::PointTime, rrule_set::RRuleSet};

fn run_test_by_vec(test_vec: Vec<(&str, Vec<&str>)>) {
    test_vec.iter().for_each(|(str, vec)| {
        assert_eq!(
            RRuleSet::from_str(str).unwrap().all(),
            vec.iter()
                .map(|time| time.parse::<PointTime>().unwrap().with_timezone(&Tz::UTC))
                .collect::<Vec<_>>()
        )
    });
}

#[test]
fn test_expand_by_day() {
    let test_vec = vec![
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3",
            vec!["20231023T180000", "20231024T180000", "20231025T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2",
            vec!["20231023T180000", "20231025T180000", "20231027T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU",
            vec!["20231023T180000", "20231031T180000", "20231106T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU;BYMONTH=1",
            vec!["20240101T180000", "20240109T180000", "20240115T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU;BYMONTHDAY=1",
            vec!["20240101T180000", "20240701T180000", "20241001T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU;BYMONTHDAY=-2",
            vec!["20241230T180000", "20250429T180000", "20260629T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU;BYYEARDAY=-1",
            vec!["20301231T180000", "20351231T180000", "20411231T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU;BYYEARDAY=2",
            vec!["20290102T180000", "20340102T180000", "20450102T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=1;BYWEEKNO=20;BYDAY=MO",
            vec!["20240513T180000", "20250512T180000", "20260511T180000"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=1;BYWEEKNO=-1;BYDAY=MO",
            vec!["20231225T180000", "20241223T180000", "20251222T180000"],
        ),
    ];
    run_test_by_vec(test_vec);
}

#[test]
fn test_expand_by_week() {
    let test_vec = vec![
        (
            "DTSTART:20231123T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;",
            vec!["20231123T180000", "20231130T180000", "20231207T180000"],
        ),
        (
            "DTSTART:20231223T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE",
            vec!["20231227T180000", "20240103T180000", "20240110T180000"],
        ),
        (
            "DTSTART:20231223T180000Z\nRRULE:FREQ=WEEKLY;COUNT=5;WKST=MO;BYDAY=WE;BYMONTH=3",
            vec!["20240306T180000", "20240313T180000", "20240320T180000", "20240327T180000", "20250305T180000"],
        ),
        (
            "DTSTART:20231223T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE;BYMONTHDAY=1",
            vec!["20240501T180000", "20250101T180000", "20251001T180000"],
        ),
        (
            "DTSTART:20231223T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE;BYYEARDAY=2",
            vec!["20300102T180000", "20360102T180000", "20410102T180000"],
        ),
        (
            "DTSTART:20231223T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE;BYYEARDAY=2;BYWEEKNO=2",
            vec![],
        ),
        (
            "DTSTART:20231223T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE;BYWEEKNO=2",
            vec!["20240110T180000", "20250108T180000", "20260107T180000"],
        ),
        (
            "DTSTART:20231224T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE;INTERVAL=2",
            vec!["20240103T180000", "20240117T180000", "20240131T180000"],
        ),
        (
            "DTSTART:20231224T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=SU;BYDAY=WE;INTERVAL=2",
            vec!["20231227T180000", "20240110T180000", "20240124T180000"],
        ),
        (
            "DTSTART:20231224T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=SU;INTERVAL=2",
            vec!["20231224T180000", "20240107T180000", "20240121T180000"],
        ),
    ];
    run_test_by_vec(test_vec);
}

#[test]
fn test_expand_by_month() {
    let test_vec = vec![
        (
            "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;",
            vec!["20231029T091800", "20231129T091800", "20231229T091800"],
        ),
        (
            "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;INTERVAL=2",
            vec!["20231029T091800", "20231229T091800", "20240229T091800"],
        ),
        (
            "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1,3",
            vec!["20231101T091800", "20231103T091800", "20231201T091800"],
        ),
        (
            "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1,3;BYDAY=FR",
            vec!["20231103T091800", "20231201T091800", "20240301T091800"],
        ),
        (
            "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1;BYDAY=1FR",
            vec!["20231201T091800", "20240301T091800", "20241101T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1;BYDAY=1FR;INTERVAL=2",
            vec!["20240301T091800", "20241101T091800", "20260501T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYDAY=2FR;",
            vec!["20231208T091800", "20240112T091800", "20240209T091800"],
        ),
        (
            "DTSTART:20231126T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYDAY=-2FR;",
            vec!["20231222T091800", "20240119T091800", "20240216T091800"],
        ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=MONTHLY;COUNT=3;INTERVAL=1;BYDAY=MO",
            vec!["20231023T180000", "20231030T180000", "20231106T180000"],
        ),
    ];
    run_test_by_vec(test_vec);
}

#[test]
fn test_set_tz_in_str() {
    let str =  "DTSTART;TZID=America/New_York:20231013T003000\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=FR;UNTIL=20231128T105959";
    let mut set = RRuleSet::from_str(str).unwrap();
    assert_eq!(set.tz, Tz::America__New_York);

    set.tz("America/Maceio");
    assert_eq!(set.tz, Tz::America__Maceio);
}
#[test]
fn test_set_until_in_str() {
    let str =  "DTSTART;TZID=America/New_York:20231013T003000\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=FR;UNTIL=20231128T105959";
    let mut set = RRuleSet::from_str(str).unwrap();
    assert_eq!(set.tz, Tz::America__New_York);

    set.set_until("20231129T105959");
    let rrule = set.rrule.get(0).unwrap();
    assert_eq!(
        rrule.until.as_ref().unwrap(),
        &"20231129T105959".parse::<PointTime>().unwrap()
    );
}
#[test]
fn test_set_between() {
    let str =  "DTSTART;TZID=America/New_York:20231013T091800\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=FR;UNTIL=20231128T105959";
    let mut set = RRuleSet::from_str(str).unwrap();

    set.set_until("20231129T105959");
    set.between("20231101T000000", "20231120T000000");
    let list: Vec<chrono::prelude::DateTime<Tz>> = set.all();
    assert_eq!(
        list,
        vec!["20231103T091800", "20231110T091800", "20231117T091800"]
            .iter()
            .map(|time| time
                .parse::<PointTime>()
                .unwrap()
                .with_timezone(&Tz::America__New_York))
            .collect::<Vec<_>>()
    );
    set.between("20231129T000000", "20231220T000000");
    assert!(set.all().is_empty());
}

#[test]
fn test_expand_by_year() {
    let test_vec = vec![
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;",
            vec!["20231123T091800", "20241123T091800", "20251123T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;INTERVAL=2",
            vec!["20231123T091800", "20251123T091800", "20271123T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;BYDAY=MO",
            vec!["20231127T091800", "20231204T091800", "20231211T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;BYDAY=MO,FR",
            vec!["20231124T091800", "20231127T091800", "20231201T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;BYDAY=-1FR",
            vec!["20231229T091800", "20241227T091800", "20251226T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;BYDAY=FR;BYMONTH=2",
            vec!["20240202T091800", "20240209T091800", "20240216T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;BYDAY=MO,FR;BYYEARDAY=20",
            vec!["20250120T091800", "20310120T091800", "20340120T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=3;WKST=MO;BYDAY=MO;BYWEEKNO=3",
            vec!["20240115T091800", "20250113T091800", "20260112T091800"],
        ),
        (
            "DTSTART:20231123T091800Z\nRRULE:FREQ=YEARLY;COUNT=30;WKST=MO;BYDAY=MO;BYWEEKNO=3;UNTIL=20260112T091700",
            vec!["20240115T091800", "20250113T091800", "20260112T091800"],
        ),
    ];
    run_test_by_vec(test_vec);
}