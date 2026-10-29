using System;
using System.Collections.Generic;
using System.Linq;

namespace FictionalWeather;

public sealed record Reading(DateTime Timestamp, double TemperatureC);

public sealed class WeatherStation
{
    public string Name { get; }
    public IReadOnlyList<Reading> Readings { get; }

    public WeatherStation(string name, IReadOnlyList<Reading> readings)
    {
        Name = name;
        Readings = readings;
    }

    public double AverageTemperature() => Readings.Average(r => r.TemperatureC);
}

public static class Program
{
    public static void Main()
    {
        var station = new WeatherStation(
            "Sundial Basin",
            new[]
            {
                new Reading(DateTime.Parse("2032-04-15T06:00:00Z"), 9.8),
                new Reading(DateTime.Parse("2032-04-15T12:00:00Z"), 17.3),
                new Reading(DateTime.Parse("2032-04-15T18:00:00Z"), 14.6),
            });

        Console.WriteLine($"{station.Name}: {station.AverageTemperature():F1} C");
    }
}

