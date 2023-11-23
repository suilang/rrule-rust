# @sl/rrule-rust.js

**Library for working with recurrence rules for calendar dates.**  


This is a rrule project written in Rust, which is ultimately packaged as WebAssembly for use. The current version of this project does not strictly adhere to the iCalendar RFC. For example, certain properties may not take effect when the recurring dimension is monthly, weekly, or daily. Additionally, the project has not yet implemented the year recurrence feature.

---

#### Client Side

```bash
$ yarn add @sl/rrule-rust
```

#### Usage

**RRuleSet:**

```es6
import init, { JsRRule, JsRRuleSet } from '@sl/rrule-rust';

// Create a rule:
init().then(() => {
  const set = new JsRRuleSet('DTSTART:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959');
  set.tz('Asia/Shanghai')

  set
    .all()
    .split(',')
    .map((str) => new Date(Number(str)))

[ "2022-05-06T10:00:00.000Z",
"2022-05-09T10:00:00.000Z",
"2022-05-10T10:00:00.000Z",
"2022-05-11T10:00:00.000Z",
"2022-05-12T10:00:00.000Z",
"2022-05-13T10:00:00.000Z",
 /* … */]
```

At this stage, you must call the `init` function before you can use the inner function. It is not supported to set the time zone directly in the string, you must use `set.tz` setting, the default is `UTC`.

Due to a communication problem, the format returned is a timestamp concatenated string, separated by commas. Need to actively divide and parse.

# test

For daily, weekly, and monthly cycles, which are basically consistent with the resolution in rrule, the following single tests are currently verified in rust.

```rust
(   "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;",
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
```