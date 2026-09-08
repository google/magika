defmodule Forecast do
  @moduledoc "Summarizes manually entered weather observations."

  defstruct [:city, :temperature_c, :condition]

  @type condition :: :sunny | :cloudy | :rain
  @type t :: %__MODULE__{
          city: String.t(),
          temperature_c: number(),
          condition: condition()
        }

  @spec new(String.t(), number(), condition()) :: t()
  def new(city, temperature_c, condition)
      when is_binary(city) and is_number(temperature_c) do
    %__MODULE__{
      city: city,
      temperature_c: temperature_c,
      condition: condition
    }
  end

  @spec warm?(t()) :: boolean()
  def warm?(%__MODULE__{temperature_c: temperature}), do: temperature >= 20

  @spec label(t()) :: String.t()
  def label(%__MODULE__{} = forecast) do
    temperature = :erlang.float_to_binary(forecast.temperature_c / 1, decimals: 1)
    "#{forecast.city}: #{temperature} C, #{forecast.condition}"
  end
end

defmodule ForecastReport do
  @spec warm_cities([Forecast.t()]) :: [String.t()]
  def warm_cities(forecasts) do
    forecasts
    |> Enum.filter(&Forecast.warm?/1)
    |> Enum.map(& &1.city)
    |> Enum.sort()
  end

  @spec count_by_condition([Forecast.t()]) :: map()
  def count_by_condition(forecasts) do
    Enum.frequencies_by(forecasts, fn forecast -> forecast.condition end)
  end

  @spec print([Forecast.t()]) :: :ok
  def print(forecasts) do
    Enum.each(forecasts, fn forecast ->
      IO.puts(Forecast.label(forecast))
    end)

    warm = warm_cities(forecasts) |> Enum.join(", ")
    IO.puts("Warm cities: #{warm}")

    count_by_condition(forecasts)
    |> Enum.sort()
    |> Enum.each(fn {condition, count} ->
      IO.puts("#{condition}: #{count}")
    end)
  end
end

forecasts = [
  Forecast.new("Auckland", 21.5, :sunny),
  Forecast.new("Christchurch", 18.0, :cloudy),
  Forecast.new("Wellington", 16.5, :rain)
]

ForecastReport.print(forecasts)
