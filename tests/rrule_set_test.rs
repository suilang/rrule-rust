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
        // (
        //     "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3",
        //     vec!["20231023T180000", "20231024T180000", "20231025T180000"],
        // ),
        // (
        //     "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2",
        //     vec!["20231023T180000", "20231025T180000", "20231027T180000"],
        // ),
        // (
        //     "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU",
        //     vec!["20231023T180000", "20231031T180000", "20231106T180000"],
        // ),
        (
            "DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=3;INTERVAL=2;BYDAY=MO,TU;BYMONTH=1",
            vec!["20240101T180000", "20240109T180000", "20240115T180000"],
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
            "DTSTART:20231123T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE",
            vec!["20231129T180000", "20231206T180000", "20231213T180000"],
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
    ];
    run_test_by_vec(test_vec);
}
