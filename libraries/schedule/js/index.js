module.exports = {
  scheduleFromSpreadsheet:      require('./table2schedule'),
  portionsFromSchedule:         require('./schedule2portions'),
  chartFromScheduleAndPortions: require('./schedule2chart'),
  spreadsheetFromSchedule:      require('./schedule2table')
}
