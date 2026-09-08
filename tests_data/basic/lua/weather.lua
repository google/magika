local observations = {
  { station = "north", temperature = 18.4 },
  { station = "south", temperature = 21.7 },
}

local function describe(observation)
  return string.format(
    "%s station: %.1f C",
    observation.station,
    observation.temperature
  )
end

for _, observation in ipairs(observations) do
  print(describe(observation))
end
