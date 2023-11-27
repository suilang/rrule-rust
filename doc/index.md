1. daily
   1. 支持interval
   2. 支持byday,但是不识别正负数
   3. 支持bymonth
   4. 支持by_month_day
   5. 支持by_year_day
   6. 不支持by_set_pos
   7. 支持by_week_no,但是性能不好，默认周一为一周的开始

2. week
   1. 支持interval
   2. 支持byday,但是不识别正负数
   3. 支持bymonth
   4. 支持by_month_day
   5. 支持by_year_day
   6. 不支持by_set_pos
   7. 支持by_week_no,但是性能不好，默认周一为一周的开始
   4. 支持wkst

3. month
   1. 支持interval
   2. 支持byday,但是不识别正负数
   3. 支持bymonth
   4. 支持by_month_day
   5. 支持by_year_day
   6. 不支持by_set_pos
   7. 支持by_week_no,但是性能不好，默认周一为一周的开始

4. year
   1. by_weekno的时候，不处理正负by_day
   2. 此场景下，如果只有by_day被设置，则by_day必须带nth，否则代表着按周循环，可使用按周循环来代替。
   3. 
bysetpos暂不实现


1. 补充无开始时间时使用当前时间解析
2. 补充对于参数有效性的判断
3. 补充可设置最大截止时间的能力
4. 补充解析json的形式
5. 梳理统一的错误提示