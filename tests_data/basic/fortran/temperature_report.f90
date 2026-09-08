program temperature_report
  implicit none

  real, dimension(4) :: readings = [18.5, 20.0, 21.5, 19.0]
  real :: mean_temperature

  mean_temperature = sum(readings) / size(readings)

  print '(A, F5.1, A)', 'Average temperature: ', mean_temperature, ' C'
end program temperature_report
