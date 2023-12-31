# @suilang/rrule

**Build with rust, 5 faster than rrule.js**

This is a rrule project written in Rust, which is ultimately packaged as WebAssembly for use. The current version of this project does not strictly adhere to the iCalendar RFC. For example, certain properties may not take effect when the recurring dimension is monthly, weekly, or daily. Additionally, the project has not yet implemented `BYHOURLY`、`BYMINUTELY` and `BYSECONDLY`.

In a standard scenario, it is 5 times faster than rrule.js. If you add timezone, it's 100 times faster.

---

## Quick Start

Just install it like a normal npm package. If you are webpack4, you may need file-loader support. For webpack5, just use it.

### Client Side

```bash
$ yarn add @suilang/rrule
```

### Server Side

Server-side calls are not currently supported, although the packaging is different, but I want to integrate it into an npm package, and I will add support for nodejs later.

### Usage

In es module, you must call the `init` function before you can use the inner function. Don't worry about performance, it may only take a few tens of milliseconds.

#### Init by string

```javascript
import init, { JsRRuleSet } from '@suilang/rrule';

init().then(() => {
  const set = new JsRRuleSet('DTSTART:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959');
  set.tz('Asia/Shanghai')

  // or with timezone in string
  // DTSTART;TZID=Asia/Shanghai:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20231121T235959

  set
    .all()
    .split(',')
    .map((str) => new Date(Number(str)))
})

[ "2022-05-06T10:00:00.000Z",
"2022-05-09T10:00:00.000Z",
"2022-05-10T10:00:00.000Z",
"2022-05-11T10:00:00.000Z",
"2022-05-12T10:00:00.000Z",
"2022-05-13T10:00:00.000Z",
 /* … */]
```

Due to a communication problem, the format returned is a timestamp concatenated string, separated by commas. Need to actively divide and parse.

You can also set start time separately in the following way:

```js
const set = new JsRRuleSet(
  "RRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959"
);

set.tz("Asia/Shanghai");
set.set_dt_start("20220506T180000Z");
```

#### Init by json

Rust does not recognize js objects directly, so the data needs to be passed in JSON form.

```javascript
import init, { getJsRRuleSet } from './pkg/rrule_rust.js';

const data = {
  dtStart: '20221126T091800Z',
  count: 3,
  freq: 'MONTHLY',
  interval: 2,
  until: '20231126T091800Z',
  wkst: 'MO',
  tz: 'America/New_York',
};
init().then(() => {
  const set = getJsRRuleSet(JSON.stringify(data));
  set
    .all()
    .split(',')
    .map((str) => new Date(Number(str)))
})

[ "2022-05-06T10:00:00.000Z",
"2022-05-09T10:00:00.000Z",
"2022-05-10T10:00:00.000Z",
"2022-05-11T10:00:00.000Z",
"2022-05-12T10:00:00.000Z",
"2022-05-13T10:00:00.000Z",
 /* … */]
```

The types of currently recognizable parameters are as follows

```javascript
interface RRuleProps {
  dtStart: string; // '20231101T120000Z'
  count: number; // 3
  freq: string; // 'DAILY'
  interval: number; // 2
  byWeekNo: number[]; // [1, 2]
  byDay: string[]; // ['MO', '-1FR']
  until: string; // '20231201T120000Z'
  wkst: string; // 'SU';
  byMonthDay: number[]; // [-1, 2]
  byMonth: number[]; // [2, 3]
  byYearDay: number[]; // [1, 50]
  tz: string; // 'America/New_York'
}
```

## Property Support

For complexity or performance reasons, I do not support all properties, some of which cause performance or understanding problems in some FREQs, and whose can be implemented in others.

The specific meanings of each parameter can be found in [rrule-js](https://www.npmjs.com/package/rrule).

I have not implemented the `HOURLY`, `MINUTELY`, and `SECONDLY` FREQ fields. The first few loops are already complicated.

The following describes the different properties supported by FREQ.

### FREQ=DAILY

- Support interval, default is 1.
- Support count, default is 65535, but if there is not have until, will return [].
- Support until，and if there is also count, stop if none is met
- Support byday, byweekday will alse recognized as byday. And positive and negative numbers are not recognized.
- Support bymonth
- Support bymonthday
- Support byyearday
- Support byweekno, However, this can cause performance problems. Also, it is mandatory to consider Monday as the beginning of the day
- Bysetpos is not supported. This field may be useful if I implement FREQ=HOYRLY. Unfortunately, that didn't happen.

### FREQ=WEEKLY

Same as `FREQ=DAILY`.It should be noted that `bysetpos` is still not supported here, although there may be several days a week to meet the conditions, but I do not understand the meaning of setting it.

### FREQ=MONTHLY

Same as `FREQ=DAILY`.

### FREQ=YEARLY

Same as `FREQ=DAILY`.

- Support byday, byweekday will alse recognized as byday. Byday can contain only nth or no nth at the same time. Actually, I don't understand why. Don't it just get the dates and filter them ?

## Need attention

1. The default and maximum cut-off time is set to 2300 years and cannot be changed at this time.
2. Because BYHOUR is not supported, the end time is compared by day. The logic will be modified later.
3. Bysetpos is not supported.

## API

#### RRuleSet.constructor

new rruleset with str.

```js
const set = new JsRRuleSet(
  "DTSTART;TZID=America/New_York:20231126T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO"
);
```

#### RRUleSet.tz

set timezone. Overwrites the value in the string.

```js
set.tz("Asia/Shanghai");
```

#### RRuleSet.set_dt_start

when use str like `RRULE:FREQ=MONTHLY;COUNT=3;WKST=MO`, without dt_start init rruleSet, You can call this function to set the start time. Overwrites the value in the string.

```js
set.set_dt_start("20231129T105959");
```

#### RRuleSet.set_until

Set until separately. Overwrites the value in the string.

```js
set.set_until("20231129T105959");
```

#### RRuleSet.set_count

Set count separately. Overwrites the value in the string.

```js
set.set_count(10);
```

#### RRuleSet.between

Used to filter the list returned by the all function. This is useful if a lot of data is returned. Filter results will include the start and end of the day. You have to deal with scenarios that return empty.

```js
set.between("20231106T091800Z", "20231130T091859Z");
```

#### RRuleSet.all

Returns all the occurrences of the rrule between `dt_start` and `until`. if set count, The maximum length of the return list is count, regardless of whether until is reached.

#### RRuleSet.valueOf

Return rrule string.

```js
const data = {
  dtStart: "20221126T091800Z",
  count: 3,
  freq: "MONTHLY",
  interval: 2,
  until: "20231126T091800Z",
  wkst: "MO",
  tz: "America/New_York",
};
const set = getJsRRuleSet(JSON.stringify(data));
console.log(set2.valueOf());

// DTSTART;TZID=America/New_York:20221126T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;UNTIL=20231126T091800Z;INTERVAL=2;WKST=MO
```

## Test

For different loops, as well as most of the various parameter combinations, I have made a comparison with rrule-js to ensure the correctness of the logic. You can view specific test cases in [there](./tests/rrule_set_test.rs).

# License

This project is licensed under [MIT License](./LICENCE.md)
