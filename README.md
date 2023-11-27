# @suilang/rrule-rust

**Library for working with recurrence rules for calendar dates.**  

This is a rrule project written in Rust, which is ultimately packaged as WebAssembly for use. The current version of this project does not strictly adhere to the iCalendar RFC. For example, certain properties may not take effect when the recurring dimension is monthly, weekly, or daily. Additionally, the project has not yet implemented `BYHOURLY`、`BYMINUTELY` and `BYSECONDLY`.

In a standard scenario, it is 5 times faster than rrule.js. If you add timezone, it's 100 times faster.

---

## Quick Start

Just install it like a normal npm package. If you are webpack4, you may need file-loader support. For webpack5, just use it.

### Client Side

```bash
$ yarn add @suilang/rrule-rust
```

### Server Side

Server-side calls are not currently supported, although the packaging is different, but I want to integrate it into an npm package, and I will add support for nodejs later.

### Usage

In es module, you must call the `init` function before you can use the inner function. Don't worry about performance, it may only take a few tens of milliseconds.

```es6
import init, { JsRRuleSet } from '@suilang/rrule-rust';

init().then(() => {
  const set = new JsRRuleSet('DTSTART:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959');
  set.tz('Asia/Shanghai')

  // or with timezone in string
  // DTSTART;Asia/Shanghai:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20231121T235959
  
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
const set = new JsRRuleSet('RRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959');

set.tz('Asia/Shanghai')
set.set_dt_start("20220506T180000Z")
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

## Test

For different loops, as well as most of the various parameter combinations, I have made a comparison with rrule-js to ensure the correctness of the logic. When I'm done with most of the rules, I'll include a github address where can view specific test cases.


# License

This project is licensed under [MIT License](./LICENCE.md)