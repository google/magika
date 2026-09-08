-module(weather_station).
-export([average/1, status/1]).

average(Readings) when is_list(Readings), Readings =/= [] ->
    lists:sum(Readings) / length(Readings).

status(Temperature) when Temperature < 5 ->
    cold;
status(Temperature) when Temperature > 28 ->
    hot;
status(_Temperature) ->
    mild.
