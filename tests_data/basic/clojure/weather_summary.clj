(ns weather-summary.core
  (:require [clojure.string :as str]))

(defrecord Reading [station temperature-c humidity])

(defn celsius->fahrenheit [temperature]
  (+ (* temperature 9/5) 32))

(defn average [values]
  (if (seq values)
    (/ (reduce + values) (count values))
    0))

(defn parse-reading [line]
  (let [[station temperature humidity] (str/split line #",")]
    (->Reading station
               (Double/parseDouble temperature)
               (Long/parseLong humidity))))

(defn station-summary [[station readings]]
  (let [temperatures (map :temperature-c readings)
        humidities (map :humidity readings)
        mean-c (average temperatures)]
    {:station station
     :samples (count readings)
     :mean-celsius (double mean-c)
     :mean-fahrenheit (double (celsius->fahrenheit mean-c))
     :mean-humidity (double (average humidities))}))

(defn summarize [lines]
  (->> lines
       (remove str/blank?)
       (map parse-reading)
       (group-by :station)
       (map station-summary)
       (sort-by :station)))

(defn format-summary [{:keys [station samples mean-celsius mean-fahrenheit
                              mean-humidity]}]
  (format "%s: %d samples, %.1f C / %.1f F, %.0f%% humidity"
          station samples mean-celsius mean-fahrenheit mean-humidity))

(def sample-readings
  ["Auckland,18.5,74"
   "Wellington,16.0,79"
   "Auckland,20.0,68"
   "Wellington,17.5,72"])

(defn -main [& _args]
  (doseq [line (map format-summary (summarize sample-readings))]
    (println line)))

(when (= *file* (System/getProperty "babashka.file"))
  (-main))
