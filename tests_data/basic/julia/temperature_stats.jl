module TemperatureStats

using Statistics

export summary_stats, celsius_to_fahrenheit, filter_freezing

"""
    celsius_to_fahrenheit(c::Float64)::Float64

Convert temperature from Celsius to Fahrenheit.
"""
function celsius_to_fahrenheit(c::Float64)::Float64
    return c * 9.0 / 5.0 + 32.0
end

"""
    filter_freezing(temps::Vector{Float64})::Vector{Float64}

Filter temperatures that are below 0.0 Celsius.
"""
function filter_freezing(temps::Vector{Float64})::Vector{Float64}
    return filter(t -> t < 0.0, temps)
end

"""
    summary_stats(temps::Vector{Float64})

Compute basic summary statistics for a temperature series.
"""
function summary_stats(temps::Vector{Float64})
    if isempty(temps)
        error("Temperature array cannot be empty")
    end
    return (
        mean = mean(temps),
        std = std(temps),
        min = minimum(temps),
        max = maximum(temps),
        count = length(temps)
    )
end

end # module
