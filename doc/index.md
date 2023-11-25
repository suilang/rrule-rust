1. daily
   1. 支持interval
   2. 不按byday
2. week
   1. 支持byday
   2. 支持interval
   3. 支持count和until，count优先
   4. 支持wkst
3. month
   1. 支持byday和bymonthday，同时存在则取交集，支持正负
   2. 支持count和until，count优先
   3. 最大检索2300年的数据

4. year
   1. by_weekno的时候，不处理正负by_day
   2. 此场景下，如果只有by_day被设置，则by_day必须带nth，否则代表着按周循环，可使用按周循环来代替。
   3. 
bysetpos暂不实现


FREQ=YEARLY
不会降级的有 by_month
强制指定的有 by_year_day by_week_no
降级为月的 by_month_day
降级为周的 BYDAY


// ByDay(Vec<NWeekday>) 过滤/获取, 有则降级为按周循环
        // ByMonthDay(Vec<i16>) 获取，已处理, 有则降级为每月几号
        // ByYearDay 获取，已处理,
        // ByWeekNo 获取，过滤,
        // ByMonth, 有则指定月
        // BySetPos,