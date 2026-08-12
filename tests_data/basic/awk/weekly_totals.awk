BEGIN {
  FS = ","
  print "region,total"
}

NR > 1 {
  totals[$1] += $3
}

END {
  for (region in totals) {
    printf "%s,%.2f\n", region, totals[region]
  }
}
