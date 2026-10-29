class WeatherReport {
    String station
    List<BigDecimal> temperatures

    BigDecimal averageTemperature() {
        temperatures.sum() / temperatures.size()
    }

    String summary() {
        def average = averageTemperature().setScale(1, BigDecimal.ROUND_HALF_UP)
        "${station}: average ${average} C across ${temperatures.size()} readings"
    }
}

def reports = [
    new WeatherReport(station: 'Glasswing Ridge', temperatures: [12.5, 17.0, 15.5]),
    new WeatherReport(station: 'Lantern Coast', temperatures: [16.0, 18.5, 17.5]),
]

reports.each { report ->
    println report.summary()
}

